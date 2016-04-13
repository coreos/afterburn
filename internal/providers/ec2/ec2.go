package ec2

import (
	"fmt"
	"net"
	"time"

	"github.com/coreos/coreos-metadata/internal/retry"
)

func FetchMetadata() (map[string]string, error) {
	public, err := fetchIP("public-ipv4")
	if err != nil {
		return nil, err
	}
	local, err := fetchIP("local-ipv4")
	if err != nil {
		return nil, err
	}
	hostname, err := fetchString("hostname")
	if err != nil {
		return nil, err
	}

	return map[string]string{
		"EC2_IPV4_LOCAL":  local.String(),
		"EC2_IPV4_PUBLIC": public.String(),
		"EC2_HOSTNAME":    hostname,
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
