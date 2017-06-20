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

package providers

import (
	"fmt"
	"net"
	"reflect"
)

type Metadata struct {
	Attributes map[string]string
	Hostname   string
	SshKeys    []string
	Network    []NetworkInterface
}

type NetworkInterface struct {
	HardwareAddress net.HardwareAddr
	Priority        int
	Nameservers     []net.IP
	IPAddresses     []net.IPNet
	Routes          []NetworkRoute
}

type NetworkRoute struct {
	Destination net.IPNet
	Gateway     net.IP
}

func (i NetworkInterface) UnitName() string {
	priority := i.Priority
	if priority == 0 {
		priority = 10
	}
	return fmt.Sprintf("%02d-%s.network", priority, i.HardwareAddress)
}

func (i NetworkInterface) NetworkConfig() string {
	config := fmt.Sprintf("[Match]\nMACAddress=%s\n\n[Network]\n", i.HardwareAddress)

	for _, nameserver := range i.Nameservers {
		config += fmt.Sprintf("DNS=%s\n", nameserver)
	}
	for _, addr := range i.IPAddresses {
		config += fmt.Sprintf("\n[Address]\nAddress=%s\n", addr.String())
	}
	for _, route := range i.Routes {
		config += fmt.Sprintf("\n[Route]\nDestination=%s\nGateway=%s\n", route.Destination.String(), route.Gateway)
	}

	return config
}

func String(s fmt.Stringer) string {
	if reflect.ValueOf(s).IsNil() {
		return ""
	}

	return s.String()
}
