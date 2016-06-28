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

package configdrive

import (
	"encoding/json"
	"fmt"
	"io/ioutil"
	"net"

	"github.com/coreos/coreos-metadata/internal/providers"
)

type jsonMap map[string]interface{}

func FetchMetadata() (providers.Metadata, error) {
	var parsedData jsonMap

	data, err := ioutil.ReadFile("/media/configdrive/ec2/2009-04-04/meta-data.json")
	if err != nil {
		return providers.Metadata{}, err
	}

	if err := json.Unmarshal(data, &parsedData); err != nil {
		return providers.Metadata{}, err
	}

	instanceId, _, err := parsedData.fetchString("instance-id")
	if err != nil {
		return providers.Metadata{}, err
	}

	public, err := parsedData.fetchIP("public-ipv4")
	if err != nil {
		return providers.Metadata{}, err
	}
	local, err := parsedData.fetchIP("local-ipv4")
	if err != nil {
		return providers.Metadata{}, err
	}
	hostname, _, err := parsedData.fetchString("hostname")
	if err != nil {
		return providers.Metadata{}, err
	}

	sshKeys, err := parsedData.fetchSshKeys()
	if err != nil {
		return providers.Metadata{}, err
	}

	return providers.Metadata{
		Attributes: map[string]string{
			"EC2_INSTANCE_ID": instanceId,
			"EC2_IPV4_LOCAL":  providers.String(local),
			"EC2_IPV4_PUBLIC": providers.String(public),
			"EC2_HOSTNAME":    hostname,
		},
		SshKeys: sshKeys,
	}, nil
}

func (c jsonMap) fetchString(key string) (string, bool, error) {
	value, ok := c[key].(string)
	return value, (ok && value != ""), nil
}

func (c jsonMap) fetchIP(key string) (net.IP, error) {
	str, present, err := c.fetchString(key)
	if err != nil {
		return nil, err
	}

	if !present {
		return nil, nil
	}

	if ip := net.ParseIP(str); ip != nil {
		return ip, nil
	} else {
		return nil, fmt.Errorf("couldn't parse %q as IP address", str)
	}
}

func (c jsonMap) fetchSshKeys() ([]string, error) {
	keys := []string{}

	keyInfoList := c["public-keys"].(map[string]interface{})
	for _, keyInfo := range keyInfoList {
		sshkey, ok := keyInfo.(map[string]interface{})["openssh-key"].(string)
		if !ok {
			return nil, nil
		}
		keys = append(keys, sshkey)
	}

	return keys, nil
}
