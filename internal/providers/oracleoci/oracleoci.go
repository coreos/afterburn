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

package oracleoci

import (
	"encoding/json"
	"strings"
	"time"

	"github.com/coreos/coreos-metadata/internal/providers"
	"github.com/coreos/coreos-metadata/internal/retry"
)

type instanceData struct {
	AvailabilityDomain string   `json:"availabilityDomain"`
	CompartmentId      string   `json:"compartmentId"`
	DisplayName        string   `json:"displayName"`
	Id                 string   `json:"id"`
	Image              string   `json:"image"`
	Region             string   `json:"region"`
	Shape              string   `json:"shape"`
	TimeCreated        uint64   `json:"timeCreated"`
	Metadata           metadata `json:"metadata"`
}

type metadata struct {
	SshAuthorizedKeys string `json:"ssh_authorized_keys"`
}

func FetchMetadata() (providers.Metadata, error) {
	instanceDataBlob, err := retry.Client{
		InitialBackoff: time.Second,
		MaxBackoff:     time.Second * 5,
		MaxAttempts:    10,
	}.Get("http://169.254.169.254/opc/v1/instance/")
	if err != nil {
		return providers.Metadata{}, err
	}

	var data instanceData
	err = json.Unmarshal(instanceDataBlob, &data)
	if err != nil {
		return providers.Metadata{}, err
	}
	return providers.Metadata{
		Attributes: map[string]string{
			"ORACLE_OCI_DISPLAY_NAME": data.DisplayName,
			"ORACLE_OCI_INSTANCE_ID":  data.Id,
			"ORACLE_OCI_REGION":       data.Region,
		},
		SshKeys: strings.Split(data.Metadata.SshAuthorizedKeys, "\n"),
	}, nil
}
