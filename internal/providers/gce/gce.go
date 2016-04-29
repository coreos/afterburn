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

package gce

import (
	"fmt"
	"net"
	"strconv"
	"strings"
	"time"

	"github.com/coreos/coreos-metadata/internal/providers"
	"github.com/coreos/coreos-metadata/internal/retry"
)

func FetchMetadata() (providers.Metadata, error) {
	public, err := fetchIP("instance/network-interfaces/0/access-configs/0/external-ip")
	if err != nil {
		return providers.Metadata{}, err
	}
	local, err := fetchIP("instance/network-interfaces/0/ip")
	if err != nil {
		return providers.Metadata{}, err
	}
	hostname, _, err := fetchString("instance/hostname")
	if err != nil {
		return providers.Metadata{}, err
	}
	sshKeys, err := fetchAllSshKeys()
	if err != nil {
		return providers.Metadata{}, err
	}

	return providers.Metadata{
		Attributes: map[string]string{
			"GCE_IP_LOCAL_0":    local.String(),
			"GCE_IP_EXTERNAL_0": public.String(),
			"GCE_HOSTNAME":      hostname,
		},
		SshKeys: sshKeys,
	}, nil
}

func fetchAllSshKeys() ([]string, error) {
	deprecatedInstanceSshKeys, err := fetchSshKeys("instance/attributes/sshKeys")
	if err != nil {
		return nil, err
	}

	if deprecatedInstanceSshKeys != nil {
		return deprecatedInstanceSshKeys, nil
	}

	instanceSshKeys, err := fetchSshKeys("instance/attributes/ssh-keys")
	if err != nil {
		return nil, err
	}

	blockProjectKeys, _, err := fetchString("instance/attributes/block-project-ssh-keys")
	if err != nil {
		return nil, err
	}

	if block, err := strconv.ParseBool(blockProjectKeys); err == nil && block {
		return instanceSshKeys, nil
	}

	projectSshKeys, err := fetchSshKeys("project/attributes/sshKeys")
	if err != nil {
		return nil, err
	}

	return append(instanceSshKeys, projectSshKeys...), nil
}

func fetchString(key string) (string, bool, error) {
	body, err := retry.Client{
		InitialBackoff: time.Second,
		MaxBackoff:     time.Second * 5,
		MaxAttempts:    10,
		Header: map[string][]string{
			"Metadata-Flavor": {"Google"},
		},
	}.Get("http://metadata.google.internal/computeMetadata/v1/" + key)

	return string(body), (body != nil), err
}

func fetchIP(key string) (net.IP, error) {
	str, _, err := fetchString(key)
	if err != nil {
		return nil, err
	}
	if ip := net.ParseIP(str); ip != nil {
		return ip, nil
	} else {
		return nil, fmt.Errorf("couldn't parse %q as IP address", str)
	}
}

func fetchSshKeys(prefix string) ([]string, error) {
	keydata, present, err := fetchString(prefix)
	if err != nil {
		return nil, fmt.Errorf("error reading keys: %v", err)
	}

	if !present {
		return nil, nil
	}

	keys := []string{}
	for _, key := range strings.Split(keydata, "\n") {
		if len(key) == 0 {
			continue
		}

		tokens := strings.SplitN(key, ":", 2)
		if len(tokens) != 2 {
			return nil, fmt.Errorf("malformed public key '%s'", key)
		}
		keys = append(keys, tokens[1])
	}

	return keys, nil
}
