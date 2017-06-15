// Copyright 2016 CoreOS, Inc.
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

package packet

import (
	"bufio"
	"encoding/json"
	"errors"
	"fmt"
	"net"
	"os"
	"strings"
	"time"

	"github.com/coreos/coreos-metadata/internal/providers"
	"github.com/coreos/coreos-metadata/internal/retry"
	"github.com/packethost/packngo/metadata"
)

func FetchMetadata() (providers.Metadata, error) {
	body, err := retry.Client{
		InitialBackoff: time.Second,
		MaxBackoff:     time.Second * 5,
		MaxAttempts:    10,
	}.Get(metadata.BaseURL + "/metadata")
	if err != nil {
		return providers.Metadata{}, err
	}

	var data struct {
		Error        string `json:"error"`
		PhoneHomeURL string `json:"phone_home_url"`
		*metadata.CurrentDevice
	}

	if err := json.Unmarshal(body, &data); err != nil {
		return providers.Metadata{}, err
	}

	if data.Error != "" {
		return providers.Metadata{}, errors.New(data.Error)
	}

	network, netdev, err := parseNetwork(data.Network)
	if err != nil {
		return providers.Metadata{}, fmt.Errorf("failed to parse network config from metadata: %v", err)
	}

	attrs := getNetworkAttrs(data.Network)
	attrs["PACKET_HOSTNAME"] = data.Hostname
	attrs["PACKET_PHONE_HOME_URL"] = data.PhoneHomeURL

	return providers.Metadata{
		Attributes: attrs,
		Hostname:   data.Hostname,
		SshKeys:    data.SSHKeys,
		Network:    network,
		NetDev:     netdev,
	}, nil
}

func parseNetwork(network metadata.NetworkInfo) ([]providers.NetworkInterface, []providers.NetworkDevice, error) {
	nameservers, err := getDNSServers()
	if err != nil {
		return nil, nil, fmt.Errorf("failed to obtain DNS servers: %v", err)
	}

	ifaces := []providers.NetworkInterface{}
	bondDev := "bond0"

	for _, iface := range network.Interfaces {
		mac, err := net.ParseMAC(iface.MAC)
		if err != nil {
			return nil, nil, fmt.Errorf("parsing MAC address %q: %v", iface.MAC, err)
		}

		ifaces = append(ifaces, providers.NetworkInterface{
			HardwareAddress: mac,
			Bond:            bondDev,
		})
	}

	iface := providers.NetworkInterface{
		Name:        bondDev,
		Priority:    5,
		Nameservers: nameservers,
	}
	for _, addr := range network.Addresses {
		addrlen := 16
		if addr.Address.To4() != nil {
			addrlen = 4
		}
		dest := net.IPNet{
			IP:   make([]byte, addrlen),
			Mask: make([]byte, addrlen),
		}
		if !addr.Public {
			if addrlen == 16 {
				// private IPv6 address??
				continue
			}
			dest = net.IPNet{
				IP:   net.IPv4(10, 0, 0, 0),
				Mask: net.IPMask(net.IPv4(255, 0, 0, 0)),
			}
		}

		iface.IPAddresses = append(iface.IPAddresses, net.IPNet{
			IP:   addr.Address,
			Mask: []byte(addr.NetworkMask),
		})
		iface.Routes = append(iface.Routes, providers.NetworkRoute{
			Destination: dest,
			Gateway:     addr.Gateway,
		})
	}
	ifaces = append(ifaces, iface)

	attrs := [][2]string{
		// yay hardcoded stuff
		{"TransmitHashPolicy", "layer3+4"},
		{"MIIMonitorSec", ".1"},
		{"UpDelaySec", ".2"},
		{"DownDelaySec", ".2"},
		{"Mode", network.BondingMode().String()},
	}
	if network.BondingMode() == metadata.BondingLACP {
		attrs = append(attrs, [2]string{"LACPTransmitRate", "fast"})
	}
	netdevs := []providers.NetworkDevice{
		{
			Name:            bondDev,
			Kind:            "bond",
			HardwareAddress: ifaces[0].HardwareAddress,
			Priority:        5,
			Sections: []providers.Section{
				{
					Name:       "Bond",
					Attributes: attrs,
				},
			},
		},
	}

	return ifaces, netdevs, nil
}

func getDNSServers() ([]net.IP, error) {
	state, err := os.Open("/run/systemd/netif/state")
	if err != nil {
		return nil, err
	}
	defer state.Close()

	var addrs []net.IP
	line := bufio.NewScanner(state)
	for line.Scan() {
		parts := strings.Split(line.Text(), "=")
		if len(parts) == 2 && parts[0] == "DNS" {
			for _, addr := range strings.Split(parts[1], " ") {
				ip := net.ParseIP(addr)
				if ip != nil {
					addrs = append(addrs, ip)
				}
			}
		}
	}
	if len(addrs) == 0 {
		return nil, fmt.Errorf("no DNS servers in %v", state.Name())
	}
	return addrs, nil
}

func getNetworkAttrs(network metadata.NetworkInfo) map[string]string {
	var publicIPv4, privateIPv4, publicIPv6, privateIPv6 []net.IP

	for _, addr := range network.Addresses {
		switch {
		case addr.Family == 4 && addr.Public:
			publicIPv4 = append(publicIPv4, addr.Address)

		case addr.Family == 4 && !addr.Public:
			privateIPv4 = append(privateIPv4, addr.Address)

		case addr.Family == 6 && addr.Public:
			publicIPv6 = append(publicIPv6, addr.Address)

		case addr.Family == 6 && !addr.Public:
			privateIPv6 = append(privateIPv6, addr.Address)
		}
	}

	addresses := make(map[string]string)

	for i, ip := range publicIPv4 {
		addresses[fmt.Sprintf("PACKET_IPV4_PUBLIC_%d", i)] = providers.String(ip)
	}

	for i, ip := range privateIPv4 {
		addresses[fmt.Sprintf("PACKET_IPV4_PRIVATE_%d", i)] = providers.String(ip)
	}

	for i, ip := range publicIPv6 {
		addresses[fmt.Sprintf("PACKET_IPV6_PUBLIC_%d", i)] = providers.String(ip)
	}

	for i, ip := range privateIPv6 {
		addresses[fmt.Sprintf("PACKET_IPV6_PRIVATE_%d", i)] = providers.String(ip)
	}

	return addresses
}
