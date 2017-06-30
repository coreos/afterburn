// Copyright 2017 CoreOS, Inc.
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

package vagrant_virtualbox

import (
	"errors"
	"fmt"
	"net"
	"os"
	"time"

	"github.com/coreos/coreos-metadata/internal/providers"
)

type metadata struct {
	privateIPv4 net.IP
	hostname    string
}

func FetchMetadata() (providers.Metadata, error) {
	config, err := genConfig()
	if err != nil {
		return providers.Metadata{}, err
	}

	return providers.Metadata{
		Attributes: map[string]string{
			"VAGRANT_VIRTUALBOX_PRIVATE_IPV4": providers.String(config.privateIPv4),
			"VAGRANT_VIRTUALBOX_HOSTNAME":     config.hostname,
		},
	}, nil
}

func genConfig() (metadata, error) {
	config := metadata{}
	var err error
	config.privateIPv4, err = getIP()
	if err != nil {
		return metadata{}, err
	}
	config.hostname, err = os.Hostname()
	if err != nil {
		return metadata{}, err
	}
	return config, nil
}

func getIP() (net.IP, error) {
	// eth1 may not yet be available or configured; wait
	maxAttempts := 30
	for attempt := 1; attempt <= maxAttempts; attempt++ {
		interfaces, err := net.Interfaces()
		if err != nil {
			return nil, err
		}
		for _, iface := range interfaces {
			if iface.Name == "eth1" {
				addrs, err := iface.Addrs()
				if err != nil {
					return nil, err
				}
				for _, addr := range addrs {
					ip, _, err := net.ParseCIDR(addr.String())
					if err != nil {
						return nil, err
					}
					if ip.To4() != nil {
						return ip, nil
					}
				}
			}
		}
		fmt.Println("eth1 not found; waiting 2 seconds")
		time.Sleep(2 * time.Second)
	}
	return nil, errors.New("eth1 was not found!")
}
