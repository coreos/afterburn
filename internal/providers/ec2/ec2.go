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

package ec2

import (
	"bufio"
	"fmt"
	"net"
	"strings"
	"time"

	"github.com/coreos/coreos-metadata/internal/providers"
	"github.com/coreos/coreos-metadata/internal/retry"
)

func FetchMetadata() (providers.Metadata, error) {
	instanceId, err := fetchString("instance-id")
	if err != nil {
		return providers.Metadata{}, err
	}

	public, err := fetchIP("public-ipv4")
	if err != nil {
		return providers.Metadata{}, err
	}
	local, err := fetchIP("local-ipv4")
	if err != nil {
		return providers.Metadata{}, err
	}
	hostname, err := fetchString("hostname")
	if err != nil {
		return providers.Metadata{}, err
	}

	sshKeys, err := fetchSshKeys()
	if err != nil {
		return providers.Metadata{}, err
	}

	return providers.Metadata{
		Attributes: map[string]string{
			"EC2_INSTANCE_ID": instanceId,
			"EC2_IPV4_LOCAL":  local.String(),
			"EC2_IPV4_PUBLIC": public.String(),
			"EC2_HOSTNAME":    hostname,
		},
		SshKeys: sshKeys,
	}, nil
}

func fetchString(key string) (string, error) {
	body, err := retry.Client{
		InitialBackoff: time.Second,
		MaxBackoff:     time.Second * 5,
		MaxAttempts:    10,
	}.Get("http://169.254.169.254/2009-04-04/meta-data/" + key)
	return string(body), err
}

func fetchIP(key string) (net.IP, error) {
	str, err := fetchString(key)
	if err != nil {
		return nil, err
	}
	if ip := net.ParseIP(str); ip != nil {
		return ip, nil
	} else {
		return nil, fmt.Errorf("couldn't parse %q as IP address", str)
	}
}

func fetchSshKeys() ([]string, error) {
	keydata, err := fetchString("public-keys")
	if err != nil {
		return nil, fmt.Errorf("error reading keys: %v", err)
	}

	scanner := bufio.NewScanner(strings.NewReader(keydata))
	keynames := []string{}
	for scanner.Scan() {
		keynames = append(keynames, scanner.Text())
	}
	if err := scanner.Err(); err != nil {
		return nil, fmt.Errorf("error parsing keys: %v", err)
	}

	keyIDs := make(map[string]string)
	for _, keyname := range keynames {
		tokens := strings.SplitN(keyname, "=", 2)
		if len(tokens) != 2 {
			return nil, fmt.Errorf("malformed public key: %q", keyname)
		}
		keyIDs[tokens[1]] = tokens[0]
	}

	keys := []string{}
	for _, id := range keyIDs {
		sshkey, err := fetchString(fmt.Sprintf("public-keys/%s/openssh-key", id))
		if err != nil {
			return nil, err
		}
		keys = append(keys, sshkey)
	}

	return keys, nil
}
