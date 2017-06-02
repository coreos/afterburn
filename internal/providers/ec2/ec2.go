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
	"encoding/json"
	"fmt"
	"net"
	"strings"
	"time"

	"github.com/coreos/coreos-metadata/internal/providers"
	"github.com/coreos/coreos-metadata/internal/retry"
)

type instanceIdDoc struct {
	PrivateIp          string `json:"privateIp"`
	DevpayProductCodes string `json:"devpayProductCodes"`
	AvailabilityZone   string `json:"availabilityZone"`
	Version            string `json:"version"`
	Region             string `json:"region"`
	PendingTime        string `json:"pendingTime"`
	InstanceId         string `json:"instanceId"`
	BillingProducts    string `json:"billingProducts"`
	InstanceType       string `json:"instanceType"`
	AccountId          string `json:"accountId"`
	Architecture       string `json:"architecture"`
	KernelId           string `json:"kernelId"`
	RamdiskId          string `json:"ramdiskId"`
	ImageId            string `json:"imageId"`
}

func FetchMetadata() (providers.Metadata, error) {
	instanceId, _, err := fetchString("meta-data/instance-id")
	if err != nil {
		return providers.Metadata{}, err
	}

	public, err := fetchIP("meta-data/public-ipv4")
	if err != nil {
		return providers.Metadata{}, err
	}
	local, err := fetchIP("meta-data/local-ipv4")
	if err != nil {
		return providers.Metadata{}, err
	}
	hostname, _, err := fetchString("meta-data/hostname")
	if err != nil {
		return providers.Metadata{}, err
	}
	availabilityZone, _, err := fetchString("meta-data/placement/availability-zone")
	if err != nil {
		return providers.Metadata{}, err
	}

	instanceIdDocBlob, _, err := fetchString("dynamic/instance-identity/document")
	if err != nil {
		return providers.Metadata{}, err
	}
	var instanceIdDoc instanceIdDoc
	err = json.Unmarshal([]byte(instanceIdDocBlob), &instanceIdDoc)
	if err != nil {
		return providers.Metadata{}, err
	}

	sshKeys, err := fetchSshKeys()
	if err != nil {
		return providers.Metadata{}, err
	}

	return providers.Metadata{
		Attributes: map[string]string{
			"EC2_INSTANCE_ID":       instanceId,
			"EC2_IPV4_LOCAL":        providers.String(local),
			"EC2_IPV4_PUBLIC":       providers.String(public),
			"EC2_HOSTNAME":          hostname,
			"EC2_AVAILABILITY_ZONE": availabilityZone,
			"EC2_REGION":            instanceIdDoc.Region,
		},
		Hostname: hostname,
		SshKeys:  sshKeys,
	}, nil
}

func fetchString(key string) (string, bool, error) {
	body, err := retry.Client{
		InitialBackoff: time.Second,
		MaxBackoff:     time.Second * 5,
		MaxAttempts:    10,
	}.Get("http://169.254.169.254/2009-04-04/" + key)
	return string(body), (body != nil), err
}

func fetchIP(key string) (net.IP, error) {
	str, present, err := fetchString(key)
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

func fetchSshKeys() ([]string, error) {
	keydata, present, err := fetchString("meta-data/public-keys")
	if err != nil {
		return nil, fmt.Errorf("error reading keys: %v", err)
	}

	if !present {
		return nil, nil
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
		sshkey, _, err := fetchString(fmt.Sprintf("meta-data/public-keys/%s/openssh-key", id))
		if err != nil {
			return nil, err
		}
		keys = append(keys, sshkey)
	}

	return keys, nil
}
