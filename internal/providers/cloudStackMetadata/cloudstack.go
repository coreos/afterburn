package cloudStackMetadata

import (
	"bufio"
	"fmt"
	"net"
	"os"
	"strings"
	"time"

	"github.com/coreos/coreos-metadata/internal/providers"
	"github.com/coreos/coreos-metadata/internal/retry"
)

const (
	LeaseRetryInterval = 500 * time.Millisecond
)

func FetchMetadata() (providers.Metadata, error) {
	m := providers.Metadata{}
	m.Attributes = make(map[string]string)

	metadata := map[string]string{
		"instance-id":       "CLOUDSTACK_INSTANCE_ID",
		"local-hostname":    "CLOUDSTACK_LOCAL_HOSTNAME",
		"public-hostname":   "CLOUDSTACK_PUBLIC_HOSTNAME",
		"availability-zone": "CLOUDSTACK_AVAILABILITY_ZONE",
		"public-ipv4":       "CLOUDSTACK_IPV4_PUBLIC",
		"local-ipv4":        "CLOUDSTACK_IPV4_LOCAL",
		"service-offering":  "CLOUDSTACK_SERVICE_OFFERING",
		"cloud-identifier":  "CLOUDSTACK_CLOUD_IDENTIFIER",
		"vm-id":             "CLOUDSTACK_VM_ID",
	}

	for key, value := range metadata {
		if err := fetchAndSet(key, value, m.Attributes); err != nil {
			return providers.Metadata{}, err
		}
	}

	if err := fetchAndSet("local-hostname", "CLOUDSTACK_HOSTNAME", m.Attributes); err != nil {
		return providers.Metadata{}, err
	}

	keys, err := fetchKeys("public-keys")
	if err != nil {
		return providers.Metadata{}, err
	}
	m.SshKeys = keys

	return m, nil
}

func fetchMetadata(key string) (string, bool, error) {
	addr, err := getDHCPServerAddress()
	if err != nil {
		return "", false, err
	}

	url := "http://" + addr + "/latest/meta-data/"
	body, err := retry.Client{
		InitialBackoff: time.Second,
		MaxBackoff:     time.Second * 5,
		MaxAttempts:    10,
	}.Get(url + key)
	return string(body), (body != nil), err
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

func fetchKeys(key string) ([]string, error) {
	keysListBlob, ok, err := fetchMetadata(key)
	if err != nil {
		return nil, err
	}
	if !ok || keysListBlob == "" {
		return nil, nil
	}
	keys := strings.Split(keysListBlob, "\n")

	return keys, nil
}

func findLease() (*os.File, error) {
	ifaces, err := net.Interfaces()
	if err != nil {
		return nil, fmt.Errorf("could not list interfaces: %v", err)
	}

	for {
		for _, iface := range ifaces {
			lease, err := os.Open(fmt.Sprintf("/run/systemd/netif/leases/%d", iface.Index))
			if os.IsNotExist(err) {
				continue
			} else if err != nil {
				return nil, err
			} else {
				return lease, nil
			}
		}

		fmt.Printf("No leases found. Waiting...")
		time.Sleep(LeaseRetryInterval)
	}
}

func getDHCPServerAddress() (string, error) {
	lease, err := findLease()
	if err != nil {
		return "", err
	}
	defer lease.Close()

	var address string
	line := bufio.NewScanner(lease)
	for line.Scan() {
		parts := strings.Split(line.Text(), "=")
		if parts[0] == "SERVER_ADDRESS" && len(parts) == 2 {
			address = parts[1]
			break
		}
	}

	if len(address) == 0 {
		return "", fmt.Errorf("dhcp server address not found in leases")
	}

	return address, nil
}
