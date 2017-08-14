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

package virtualbox

import (
	"os"

	"github.com/coreos/coreos-metadata/internal/providers"
)

type metadata struct {
	hostname string
}

func FetchMetadata() (providers.Metadata, error) {
	config, err := genConfig()
	if err != nil {
		return providers.Metadata{}, err
	}

	return providers.Metadata{
		Attributes: map[string]string{
			"VIRTUALBOX_HOSTNAME": config.hostname,
		},
	}, nil
}

func genConfig() (metadata, error) {
	config := metadata{}
	var err error
	config.hostname, err = os.Hostname()
	if err != nil {
		return metadata{}, err
	}
	return config, nil
}
