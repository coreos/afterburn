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

package main

import (
	"errors"
	"flag"
	"fmt"
	"io/ioutil"
	"os"
	"os/user"
	"path/filepath"
	"strings"

	"github.com/coreos/coreos-metadata/internal/providers"
	"github.com/coreos/coreos-metadata/internal/providers/azure"
	"github.com/coreos/coreos-metadata/internal/providers/digitalocean"
	"github.com/coreos/coreos-metadata/internal/providers/ec2"
	"github.com/coreos/coreos-metadata/internal/providers/gce"
	"github.com/coreos/coreos-metadata/internal/providers/openstackMetadata"
	"github.com/coreos/coreos-metadata/internal/providers/packet"
	"github.com/coreos/coreos-metadata/internal/providers/vagrant_virtualbox"

	"github.com/coreos/update-ssh-keys/authorized_keys_d"
)

var (
	version       = "was not built properly"
	versionString = fmt.Sprintf("coreos-metadata %s", version)

	ErrUnknownProvider = errors.New("unknown provider")
)

const (
	cmdlinePath    = "/proc/cmdline"
	cmdlineOEMFlag = "coreos.oem.id"
)

func main() {
	flags := struct {
		attributes   string
		cmdline      bool
		hostname     string
		networkUnits string
		provider     string
		sshKeys      string
		version      bool
	}{}

	flag.StringVar(&flags.attributes, "attributes", "", "The file into which the metadata attributes are written")
	flag.BoolVar(&flags.cmdline, "cmdline", false, "Read the cloud provider from the kernel cmdline")
	flag.StringVar(&flags.hostname, "hostname", "", "The file into which the hostname should be written")
	flag.StringVar(&flags.networkUnits, "network-units", "", "The directory into which network units are written")
	flag.StringVar(&flags.provider, "provider", "", "The name of the cloud provider")
	flag.StringVar(&flags.sshKeys, "ssh-keys", "", "Update SSH keys for the given user")
	flag.BoolVar(&flags.version, "version", false, "Print the version and exit")

	flag.Parse()

	if flags.version {
		fmt.Println(versionString)
		return
	}

	if flags.cmdline && flags.provider == "" {
		args, err := ioutil.ReadFile(cmdlinePath)
		if err != nil {
			fmt.Fprintf(os.Stderr, "could not read cmdline: %v\n", err)
			os.Exit(2)
		}

		flags.provider = parseCmdline(args)
	}

	metadataFn, err := getMetadataProvider(flags.provider)
	if err != nil {
		fmt.Fprintf(os.Stderr, "invalid provider %q\n", flags.provider)
		os.Exit(2)
	}

	metadata, err := metadataFn()
	if err != nil {
		fmt.Fprintf(os.Stderr, "failed to fetch metadata: %v\n", err)
		os.Exit(1)
	}

	if err := writeMetadataAttributes(flags.attributes, metadata); err != nil {
		fmt.Fprintf(os.Stderr, "failed to write metadata attributes: %v\n", err)
		os.Exit(1)
	}

	if err := writeMetadataKeys(flags.sshKeys, metadata); err != nil {
		fmt.Fprintf(os.Stderr, "failed to write metadata keys: %v\n", err)
		os.Exit(1)
	}

	if err := writeHostname(flags.hostname, metadata); err != nil {
		fmt.Fprintf(os.Stderr, "failed to write hostname: %v\n", err)
		os.Exit(1)
	}

	if err := writeNetworkUnits(flags.networkUnits, metadata); err != nil {
		fmt.Fprintf(os.Stderr, "failed to write network units: %v\n", err)
		os.Exit(1)
	}
}

func parseCmdline(cmdline []byte) (oem string) {
	for _, arg := range strings.Split(string(cmdline), " ") {
		parts := strings.SplitN(strings.TrimSpace(arg), "=", 2)
		key := parts[0]

		if key != cmdlineOEMFlag {
			continue
		}

		if len(parts) == 2 {
			oem = parts[1]
		}
	}

	return
}

func getMetadataProvider(providerName string) (func() (providers.Metadata, error), error) {
	switch providerName {
	case "azure":
		return azure.FetchMetadata, nil
	case "digitalocean":
		return digitalocean.FetchMetadata, nil
	case "ec2":
		return ec2.FetchMetadata, nil
	case "gce":
		return gce.FetchMetadata, nil
	case "packet":
		return packet.FetchMetadata, nil
	case "openstack-metadata":
		return openstackMetadata.FetchMetadata, nil
	case "vagrant-virtualbox":
		return vagrant_virtualbox.FetchMetadata, nil
	default:
		return nil, ErrUnknownProvider
	}
}

func writeVariable(out *os.File, key string, value string) (err error) {
	if len(value) > 0 {
		_, err = fmt.Fprintf(out, "COREOS_%s=%s\n", key, value)
	}
	return
}

func writeMetadataAttributes(attributes string, metadata providers.Metadata) error {
	if attributes == "" {
		return nil
	}

	if err := os.MkdirAll(filepath.Dir(attributes), 0755); err != nil {
		fmt.Fprintf(os.Stderr, "failed to create directory: %v\n", err)
		os.Exit(1)
	}

	out, err := os.Create(attributes)
	if err != nil {
		fmt.Fprintf(os.Stderr, "failed to create file: %v\n", err)
		os.Exit(1)
	}
	defer out.Close()

	for key, value := range metadata.Attributes {
		if err := writeVariable(out, key, value); err != nil {
			return err
		}
	}
	return nil
}

func writeMetadataKeys(username string, metadata providers.Metadata) error {
	if username == "" || metadata.SshKeys == nil {
		return nil
	}

	usr, err := user.Lookup(username)
	if err != nil {
		return fmt.Errorf("unable to lookup user %q: %v", username, err)
	}

	akd, err := authorized_keys_d.Open(usr, true)
	if err != nil {
		return err
	}
	defer akd.Close()

	ks := strings.Join(metadata.SshKeys, "\n") + "\n"
	if err := akd.Add("coreos-metadata", []byte(ks), true, true); err != nil {
		return err
	}

	return akd.Sync()
}

func writeHostname(path string, metadata providers.Metadata) error {
	if path == "" || metadata.Hostname == "" {
		return nil
	}

	err := os.MkdirAll(filepath.Dir(path), 0755)
	if err != nil {
		return err
	}

	return ioutil.WriteFile(path, []byte(metadata.Hostname+"\n"), 0644)
}

func writeNetworkUnits(root string, metadata providers.Metadata) error {
	if root == "" || metadata.Network == nil {
		return nil
	}

	err := os.MkdirAll(root, 0755)
	if err != nil {
		return err
	}

	for _, iface := range metadata.Network {
		name := filepath.Join(root, fmt.Sprintf("00-%s.network", iface.HardwareAddress))
		err := ioutil.WriteFile(name, []byte(iface.NetworkConfig()), 0644)
		if err != nil {
			return err
		}
	}

	return nil
}
