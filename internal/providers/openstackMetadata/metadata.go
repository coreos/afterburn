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

package openstackMetadata

import (
	"fmt"
	"path"
	"strings"
	"time"

	"github.com/coreos/coreos-metadata/internal/providers"
	"github.com/coreos/coreos-metadata/internal/retry"
)

const (
	metadataEndpoint = "http://169.254.169.254/latest/meta-data/"
)

func FetchMetadata() (providers.Metadata, error) {
	m := providers.Metadata{}
	m.Attributes = make(map[string]string)

	if err := fetchAndSet("instance-id", "OPENSTACK_INSTANCE_ID", m.Attributes); err != nil {
		return providers.Metadata{}, err
	}

	if err := fetchAndSet("local-ipv4", "OPENSTACK_IPV4_LOCAL", m.Attributes); err != nil {
		return providers.Metadata{}, err
	}

	if err := fetchAndSet("public-ipv4", "OPENSTACK_IPV4_PUBLIC", m.Attributes); err != nil {
		return providers.Metadata{}, err
	}

	if err := fetchAndSet("hostname", "OPENSTACK_HOSTNAME", m.Attributes); err != nil {
		return providers.Metadata{}, err
	}

	keys, err := fetchKeys()
	if err != nil {
		return providers.Metadata{}, err
	}
	m.SshKeys = keys

	return m, nil
}

func fetchAndSet(key, attrKey string, attributes map[string]string) error {
	val, ok, err := fetchMetadata(key)
	if err != nil {
		return err
	}
	if !ok || val == "" {
		return nil
	}
	attributes[attrKey] = val
	return nil
}

func fetchKeys() ([]string, error) {
	keysListBlob, ok, err := fetchMetadata("public-keys")
	if err != nil {
		return nil, err
	}
	if !ok || keysListBlob == "" {
		return nil, nil
	}
	keysList := strings.Split(keysListBlob, "\n")

	var keys []string

	if len(keysList) > 0 {
		keyID := keysList[0]
		keyTokens := strings.Split(keyID, "=")
		if len(keyTokens) != 2 {
			return nil, fmt.Errorf("error parsing keyID %s", keyID)
		}
		keyNum := keyTokens[0]
		// keyTokens[1] is the name of the key, but is currently unused here
		key, ok, err := fetchMetadata(path.Join("public-keys", keyNum, "openssh-key"))
		if err != nil {
			return nil, err
		}
		if !ok || key == "" {
			return nil, fmt.Errorf("problem fetching key %s", keyID)
		}
		keys = append(keys, key)
	}
	return keys, nil
}

func fetchMetadata(key string) (string, bool, error) {
	body, err := retry.Client{
		InitialBackoff: time.Second,
		MaxBackoff:     time.Second * 5,
		MaxAttempts:    10,
	}.Get(metadataEndpoint + key)
	return string(body), (body != nil), err
}
