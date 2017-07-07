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

package cloudStackConfigDrive

import (
	"fmt"
	"path/filepath"
	"strings"

	"errors"
	"github.com/coreos/coreos-metadata/internal/providers"
	"io/ioutil"
	"os"
	"os/exec"
	"syscall"
)

const (
	diskByLabelPath               = "/dev/disk/by-label/"
	configDriveMetadataPath       = "/cloudstack/metadata/"
	configDriveMetadataMountPoint = "/media/ConfigDrive/cloudstack/metadata/"
)

func FetchMetadata() (providers.Metadata, error) {
	m := providers.Metadata{}
	m.Attributes = make(map[string]string)
	mnt := ""

	if !fileExists(configDriveMetadataMountPoint) {
		err := error(nil)
		mnt, err = mountConfigDrive("config-2")
		if err != nil {
			if mnt, err = mountConfigDrive("CONFIG-2"); err != nil {
				return m, err
			}
		}
		defer unmountConfigDrive(mnt)
	} else {
		mnt = configDriveMetadataMountPoint
	}

	if err := fetchAndSet("availability_zone.txt", mnt, "CLOUDSTACK_AVAILABILITY_ZONE", m.Attributes); err != nil {
		return providers.Metadata{}, err
	}

	if err := fetchAndSet("instance_id.txt", mnt, "CLOUDSTACK_INSTANCE_ID", m.Attributes); err != nil {
		return providers.Metadata{}, err
	}

	if err := fetchAndSet("service_offering.txt", mnt, "CLOUDSTACK_SERVICE_OFFERING", m.Attributes); err != nil {
		return providers.Metadata{}, err
	}

	if err := fetchAndSet("cloud_identifier.txt", mnt, "CLOUDSTACK_CLOUD_IDENTIFIER", m.Attributes); err != nil {
		return providers.Metadata{}, err
	}

	if err := fetchAndSet("local_hostname.txt", mnt, "CLOUDSTACK_LOCAL_HOSTNAME", m.Attributes); err != nil {
		return providers.Metadata{}, err
	}

	if err := fetchAndSet("vm_id.txt", mnt, "CLOUDSTACK_VM_ID", m.Attributes); err != nil {
		return providers.Metadata{}, err
	}

	keys, err := fetchKeys(mnt)
	if err != nil {
		return providers.Metadata{}, err
	}
	m.SshKeys = keys
	return m, nil
}

func fileExists(path string) bool {
	_, err := os.Stat(path)
	return (err == nil)
}

func labelExists(label string) bool {
	_, err := getPath(label)
	return (err == nil)
}

func getPath(label string) (string, error) {
	path := diskByLabelPath + label

	if fileExists(path) {
		return path, nil
	}

	return "", fmt.Errorf("label not found: %s", label)
}

func mountConfigDrive(label string) (string, error) {
	if !labelExists(label) {
		return "", errors.New("Not able to find config drive.")
	}
	path, err := getPath(label)

	mnt, err := ioutil.TempDir("", "coreos-metadata")
	if err != nil {
		return "", fmt.Errorf("failed to create temp directory: %v", err)
	}

	cmd := exec.Command("/bin/mount", "-o", "ro", "-t", "auto", path, mnt)
	if err := cmd.Run(); err != nil {
		return "", err
	}

	return mnt, nil
}

func readFromConfigDrive(mnt string, file string) (string, bool, error) {
	configDrivePath := filepath.Join(mnt, configDriveMetadataPath, file)

	if !fileExists(configDrivePath) {
		return "", false, nil
	}

	output, err := ioutil.ReadFile(configDrivePath)

	if err != nil {
		return "", false, err
	}

	return string(output), true, nil
}

func unmountConfigDrive(mnt string) {
	defer os.Remove(mnt)
	defer syscall.Unmount(mnt, 0)
}

func fetchAndSet(key, mnt string, attrKey string, attributes map[string]string) error {
	val, ok, err := readFromConfigDrive(mnt, key)
	if err != nil {
		return err
	}
	if !ok || val == "" {
		return nil
	}
	attributes[attrKey] = val
	return nil
}

func fetchKeys(mnt string) ([]string, error) {
	keysListBlob, ok, err := readFromConfigDrive(mnt, "public_keys.txt")
	if err != nil {
		return nil, err
	}
	if !ok || keysListBlob == "" {
		return nil, nil
	}
	keys := strings.Split(keysListBlob, "\n")

	return keys, nil
}
