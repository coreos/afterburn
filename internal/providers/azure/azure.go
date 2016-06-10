// Copyright 2015 CoreOS, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package azure

import (
	"encoding/xml"
	"fmt"
	"net"
	"strconv"
	"time"

	"github.com/coreos/coreos-metadata/internal/providers"
	"github.com/coreos/coreos-metadata/internal/retry"

	"github.com/godbus/dbus"
	"github.com/godbus/dbus/introspect"
)

const (
	AgentName             = "com.coreos.metadata"
	FabricProtocolVersion = "2012-11-30"
)

type metadata struct {
	virtualIPv4 net.IP
	dynamicIPv4 net.IP
}

func FetchMetadata() (providers.Metadata, error) {
	addr, err := getFabricAddress()
	if err != nil {
		return providers.Metadata{}, err
	}

	if err := assertFabricCompatible(addr, FabricProtocolVersion); err != nil {
		return providers.Metadata{}, err
	}

	config, err := fetchSharedConfig(addr)
	if err != nil {
		return providers.Metadata{}, err
	}

	return providers.Metadata{
		Attributes: map[string]string{
			"AZURE_IPV4_DYNAMIC": providers.String(config.dynamicIPv4),
			"AZURE_IPV4_VIRTUAL": providers.String(config.virtualIPv4),
		},
	}, nil
}

func getClient() retry.Client {
	client := retry.Client{
		InitialBackoff: time.Second,
		MaxBackoff:     time.Second * 5,
		MaxAttempts:    10,
		Header: map[string][]string{
			"x-ms-agent-name": {AgentName},
			"x-ms-version":    {FabricProtocolVersion},
			"Content-Type":    {"text/xml; charset=utf-8"},
		},
	}

	return client
}

/* godbus doesn't have a way to escape things. This will escape leading numbers, which is all we need */
func escapeNumber(num int) string {
	return "_3" + strconv.FormatUint(uint64(num), 10)
}

func getFabricAddress() (net.IP, error) {
	conn, err := dbus.SystemBus()
	if err != nil {
		return nil, err
	}

	ifaces, err := net.Interfaces()
	if err != nil {
		return nil, fmt.Errorf("could not list interfaces: %v", err)
	}

	var data []byte

	for _, link := range ifaces {
		//first introspect the link to ensure it has a lease child
		linkObjPath := dbus.ObjectPath("/org/freedesktop/network1/link/" + escapeNumber(link.Index))
		node, err := introspect.Call(conn.Object("org.freedesktop.network1", linkObjPath))

		if err != nil {
			return nil, err
		}

		hasLease := false
		for _, child := range node.Children {
			if child.Name == "lease" {
				hasLease = true
				break
			}
		}

		if !hasLease {
			continue
		}

		//then get the data from the lease, if we found it
		linkObjPath = dbus.ObjectPath(string(linkObjPath) + "/lease")
		dbusLeaseObj := conn.Object("org.freedesktop.network1", linkObjPath)
		rawopts, err := dbusLeaseObj.GetProperty("org.freedesktop.network1.Link.Lease.PrivateOptions")

		if err != nil {
			return nil, err
		}

		if rawopts.Signature().String() == "a{yay}" {
			options := rawopts.Value().(map[byte][]byte)
			if options[245] != nil {
				data = options[245]
				break
			}
		}
	}

	if data == nil {
		return nil, fmt.Errorf("Did not find an lease with option 245")
	}

	return net.IPv4(data[0], data[1], data[2], data[3]), nil
}

func assertFabricCompatible(endpoint net.IP, desiredVersion string) error {
	body, err := getClient().Getf("http://%s/?comp=versions", endpoint)
	if err != nil {
		return fmt.Errorf("failed to fetch versions: %v", err)
	}

	versions := struct {
		Supported struct {
			Versions []string `xml:"Version"`
		}
		Preferred struct {
			Version string
		}
	}{}

	if err := xml.Unmarshal(body, &versions); err != nil {
		return fmt.Errorf("failed to unmarshal response: %v", err)
	}

	for _, version := range versions.Supported.Versions {
		if version == desiredVersion {
			return nil
		}
	}

	return fmt.Errorf("fabric version %s is not compatible", desiredVersion)
}

func fetchSharedConfig(endpoint net.IP) (metadata, error) {
	client := getClient()

	body, err := client.Getf("http://%s/machine/?comp=goalstate", endpoint)
	if err != nil {
		return metadata{}, fmt.Errorf("failed to fetch goal state: %v", err)
	}

	goal := struct {
		Container struct {
			RoleInstanceList struct {
				RoleInstance struct {
					Configuration struct {
						SharedConfig string
					}
				}
			}
		}
	}{}

	if err := xml.Unmarshal(body, &goal); err != nil {
		return metadata{}, fmt.Errorf("failed to unmarshal response: %v", err)
	}

	body, err = client.Get(goal.Container.RoleInstanceList.RoleInstance.Configuration.SharedConfig)
	if err != nil {
		return metadata{}, fmt.Errorf("failed to fetch shared config: %v", err)
	}

	sharedConfig := struct {
		Incarnation struct {
			Instance string `xml:"instance,attr"`
		}
		Instances struct {
			Instances []struct {
				Id             string `xml:"id,attr"`
				Address        string `xml:"address,attr"`
				InputEndpoints struct {
					Endpoints []struct {
						LoadBalancedPublicAddress string `xml:"loadBalancedPublicAddress,attr"`
					} `xml:"Endpoint"`
				}
			} `xml:"Instance"`
		}
	}{}

	if err := xml.Unmarshal(body, &sharedConfig); err != nil {
		return metadata{}, err
	}

	config := metadata{}
	for _, i := range sharedConfig.Instances.Instances {
		if i.Id == sharedConfig.Incarnation.Instance {
			config.dynamicIPv4 = net.ParseIP(i.Address)

			for _, e := range i.InputEndpoints.Endpoints {
				host, _, err := net.SplitHostPort(e.LoadBalancedPublicAddress)
				if err == nil {
					config.virtualIPv4 = net.ParseIP(host)
					break
				}
			}

			break
		}
	}

	return config, nil
}
