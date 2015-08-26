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
	"flag"
	"fmt"
	"io/ioutil"
	"net"
	"net/http"
	"os"
	"path"
	"time"
)

var (
	version       = "was not built properly"
	versionString = fmt.Sprintf("coreos-metadata %s", version)
)

type metadata struct {
	PublicIPv4 net.IP
	LocalIPv4  net.IP
	Hostname   string
}

type retryClient struct {
	InitialBackoff time.Duration
	MaxBackoff     time.Duration
	MaxAttempts    int
}

func (c retryClient) Get(url string) ([]byte, error) {
	delay := c.InitialBackoff
	for attempt := 1; attempt <= c.MaxAttempts; attempt++ {
		fmt.Printf("fetching %q: attempt #%d\n", url, attempt)

		if response, err := http.Get(url); err != nil {
			fmt.Printf("failed to fetch: %v\n", err)
		} else if response.StatusCode != http.StatusOK {
			fmt.Printf("failed to fetch: %s\n", http.StatusText(response.StatusCode))
		} else {
			defer response.Body.Close()
			return ioutil.ReadAll(response.Body)
		}

		time.Sleep(delay)
		delay *= 2
		if delay > c.MaxBackoff {
			delay = c.MaxBackoff
		}
	}

	return nil, fmt.Errorf("timed out while fetching %q", url)
}

func main() {
	flags := struct {
		provider string
		output   string
		version  bool
	}{}

	flag.StringVar(&flags.provider, "provider", "", "The name of the cloud provider")
	flag.StringVar(&flags.output, "output", "", "The file into which the metadata is written")
	flag.BoolVar(&flags.version, "version", false, "Print the version and exit")

	flag.Parse()

	if flags.version {
		fmt.Println(versionString)
		return
	}

	switch flags.provider {
	case "ec2":
	default:
		fmt.Fprintf(os.Stderr, "invalid provider %q\n", flags.provider)
		os.Exit(2)
	}

	if err := os.MkdirAll(path.Dir(flags.output), 0755); err != nil {
		fmt.Fprintf(os.Stderr, "failed to create directory: %v\n", err)
		os.Exit(1)
	}

	out, err := os.Create(flags.output)
	if err != nil {
		fmt.Fprintf(os.Stderr, "failed to create file: %v\n", err)
		os.Exit(1)
	}
	defer out.Close()

	if metadata, err := fetchMetadata(flags.provider); err == nil {
		if err := writeMetadata(out, metadata); err != nil {
			fmt.Fprintf(os.Stderr, "failed to write metadata: %v\n", err)
			os.Exit(1)
		}
	} else {
		fmt.Fprintf(os.Stderr, "failed to fetch metadata: %v\n", err)
		os.Exit(1)
	}
}

func fetchString(url string) (string, error) {
	body, err := retryClient{
		InitialBackoff: time.Second,
		MaxBackoff:     time.Second * 5,
		MaxAttempts:    10,
	}.Get(url)
	return string(body), err
}

func fetchIP(url string) (net.IP, error) {
	str, err := fetchString(url)
	if err != nil {
		return nil, err
	}
	if ip := net.ParseIP(str); ip != nil {
		return ip, nil
	} else {
		return nil, fmt.Errorf("couldn't parse %q as IP address", str)
	}
}

func fetchMetadata(provider string) (metadata, error) {
	switch provider {
	case "ec2":
		public, err := fetchIP("http://169.254.169.254/2009-04-04/meta-data/public-ipv4")
		if err != nil {
			return metadata{}, err
		}
		local, err := fetchIP("http://169.254.169.254/2009-04-04/meta-data/local-ipv4")
		if err != nil {
			return metadata{}, err
		}
		hostname, err := fetchString("http://169.254.169.254/2009-04-04/meta-data/hostname")
		if err != nil {
			return metadata{}, err
		}

		return metadata{
			PublicIPv4: public,
			LocalIPv4:  local,
			Hostname:   hostname,
		}, nil
	default:
		panic("bad provider")
	}
}

func writeIPVariable(out *os.File, key string, value net.IP) error {
	if len(value) == 0 {
		return nil
	}
	return writeVariable(out, key, value)
}

func writeStringVariable(out *os.File, key, value string) error {
	if len(value) == 0 {
		return nil
	}
	return writeVariable(out, key, value)
}

func writeVariable(out *os.File, key string, value interface{}) error {
	_, err := fmt.Fprintf(out, "%s=%s\n", key, value)
	return err
}

func writeMetadata(out *os.File, metadata metadata) error {
	if err := writeIPVariable(out, "COREOS_IPV4_PUBLIC", metadata.PublicIPv4); err != nil {
		return err
	}
	if err := writeIPVariable(out, "COREOS_IPV4_LOCAL", metadata.LocalIPv4); err != nil {
		return err
	}
	if err := writeStringVariable(out, "COREOS_HOSTNAME", metadata.Hostname); err != nil {
		return err
	}
	return nil
}
