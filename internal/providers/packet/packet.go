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
	"encoding/json"
	"errors"
	"fmt"
	"net"
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
		Error string `json:"error"`
		*metadata.CurrentDevice
	}

	if err := json.Unmarshal(body, &data); err != nil {
		return providers.Metadata{}, err
	}

	if data.Error != "" {
		return providers.Metadata{}, errors.New(data.Error)
	}

	attrs := ipAddresses(data.Network)
	attrs["PACKET_HOSTNAME"] = data.Hostname

	return providers.Metadata{
		Attributes: attrs,
		Hostname:   data.Hostname,
		SshKeys:    data.SSHKeys,
	}, nil
}

func ipAddresses(network metadata.NetworkInfo) map[string]string {
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
