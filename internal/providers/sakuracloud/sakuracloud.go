// Copyright 2017 SAKURA Internet, Inc.
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

package sakuracloud

import (
	"encoding/json"
	"fmt"
	"io/ioutil"
	"net"

	"github.com/coreos/coreos-metadata/internal/providers"
)

const (
	metadataFile = "/usr/share/oem/metadata.json"
)

type Address struct {
	IPAddress string `json:"ip_address"`
	Netmask   string `json:"netmask"`
	Cidr      int    `json:"cidr"`
	Gateway   string `json:"gateway"`
}

type Interface struct {
	IPv4 *Address `json:"ipv4"`
	IPv6 *Address `json:"ipv6"`
	MAC  string   `json:"mac"`
}

type DNS struct {
	Nameservers []string `json:"nameservers"`
}

type Metadata struct {
	Hostname   string      `json:"hostname"`
	Interfaces []Interface `json:"interfaces"`
	PublicKeys []string    `json:"public_keys"`
	DNS        DNS         `json:"dns"`
}

func FetchMetadata() (providers.Metadata, error) {
	rawConfig, err := ioutil.ReadFile(metadataFile)

	if err != nil {
		return providers.Metadata{}, fmt.Errorf("failed to read metadata: %v", err)
	}

	var m Metadata
	if err = json.Unmarshal(rawConfig, &m); err != nil {
		return providers.Metadata{}, fmt.Errorf("failed to unmarshal metadata: %v", err)
	}

	network, err := parseNetwork(m)
	if err != nil {
		return providers.Metadata{}, fmt.Errorf("failed to parse network config from metadata: %v", err)
	}

	return providers.Metadata{
		Attributes: parseAttributes(m),
		Hostname:   m.Hostname,
		Network:    network,
		SshKeys:    m.PublicKeys,
	}, nil
}

func parseAttributes(metadata Metadata) map[string]string {
	attrs := map[string]string{
		"SAKURACLOUD_HOSTNAME": metadata.Hostname,
	}

	for i, iface := range metadata.Interfaces {
		if iface.IPv4 != nil {
			attrs[fmt.Sprintf("SAKURACLOUD_IPV4_%d", i)] =
				providers.String(net.ParseIP(iface.IPv4.IPAddress))
		}
		if iface.IPv6 != nil {
			attrs[fmt.Sprintf("SAKURACLOUD_IPV6_%d", i)] =
				providers.String(net.ParseIP(iface.IPv6.IPAddress))
		}
	}

	return attrs
}

func parseNetwork(metadata Metadata) ([]providers.NetworkInterface, error) {
	servers, err := parseNameservers(metadata.DNS.Nameservers)
	if err != nil {
		return nil, err
	}

	ifaces := make([]providers.NetworkInterface, len(metadata.Interfaces))
	for i, iface := range metadata.Interfaces {
		mac, err := net.ParseMAC(iface.MAC)
		if err != nil {
			return nil, fmt.Errorf("could not parse %q as MAC address", iface.MAC)
		}
		addrs, routes, err := parseInterface(iface)
		if err != nil {
			return nil, err
		}

		ifaces[i] = providers.NetworkInterface{
			HardwareAddress: mac,
			Nameservers:     servers,
			IPAddresses:     addrs,
			Routes:          routes,
		}
	}

	return ifaces, nil
}

func parseInterface(iface Interface) ([]net.IPNet, []providers.NetworkRoute, error) {
	var addrs []net.IPNet
	var routes []providers.NetworkRoute

	if iface.IPv4 != nil {
		addr, err := parseIPv4Address(*iface.IPv4)
		if err != nil {
			return nil, nil, err
		}
		addrs = append(addrs, addr)

		route, err := parseRoute(iface.IPv4.Gateway, addr)
		if err != nil {
			return nil, nil, err
		}

		routes = append(routes, providers.NetworkRoute{
			Destination: net.IPNet{
				IP:   net.IPv4zero,
				Mask: net.IPMask(net.IPv4zero),
			},
			Gateway: route.Gateway,
		})
	}
	if iface.IPv6 != nil {
		addr, err := parseIPv6Address(*iface.IPv6)
		if err != nil {
			return nil, nil, err
		}
		addrs = append(addrs, addr)

		route, err := parseRoute(iface.IPv6.Gateway, addr)
		if err != nil {
			return nil, nil, err
		}

		routes = append(routes, providers.NetworkRoute{
			Destination: net.IPNet{
				IP:   net.IPv6zero,
				Mask: net.IPMask(net.IPv6zero),
			},
			Gateway: route.Gateway,
		})
	}

	return addrs, routes, nil
}

func parseIPv4Address(address Address) (net.IPNet, error) {
	ip := net.ParseIP(address.IPAddress)
	if ip == nil {
		return net.IPNet{}, fmt.Errorf("could not parse %q as IPv4 address", address.IPAddress)
	}

	mask := net.ParseIP(address.Netmask)
	if mask == nil {
		return net.IPNet{}, fmt.Errorf("could not parse %q as IPv4 mask", address.Netmask)
	}

	return net.IPNet{
		IP:   ip,
		Mask: net.IPMask(mask.To4()),
	}, nil
}

func parseIPv6Address(address Address) (net.IPNet, error) {
	ip := net.ParseIP(address.IPAddress)
	if ip == nil {
		return net.IPNet{}, fmt.Errorf("could not parse %q as IPv6 address", address.IPAddress)
	}

	return net.IPNet{
		IP:   ip,
		Mask: net.CIDRMask(address.Cidr, net.IPv6len*8),
	}, nil
}

func parseRoute(gateway string, destination net.IPNet) (providers.NetworkRoute, error) {
	addr := net.ParseIP(gateway)
	if addr == nil {
		return providers.NetworkRoute{}, fmt.Errorf("could not parse %q as gateway address", gateway)
	}

	return providers.NetworkRoute{
		Destination: destination,
		Gateway:     addr,
	}, nil
}

func parseNameservers(servers []string) ([]net.IP, error) {
	var addrs []net.IP
	for _, server := range servers {
		addr := net.ParseIP(server)
		if addr == nil {
			return nil, fmt.Errorf("could not parse %q as IP address", server)
		}

		addrs = append(addrs, addr)
	}
	return addrs, nil
}
