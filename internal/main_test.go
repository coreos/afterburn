package main

import (
	"errors"
	"reflect"
	"testing"
)

func TestGetMetadataProvider(t *testing.T) {
	tests := []struct {
		desc string
		name string
		err  error
	}{
		{
			desc: "supported provider",
			name: "digitalocean",
			err:  nil,
		},
		{
			desc: "unknown provider",
			name: "not-supported",
			err:  errors.New("unknown provider"),
		},
		{
			desc: "empty provider",
			name: "",
			err:  errors.New("unknown provider"),
		},
	}

	for _, tt := range tests {
		_, err := getMetadataProvider(tt.name)
		if !reflect.DeepEqual(err, tt.err) {
			t.Errorf("%s:\nwant: %v\n got: %v", tt.desc, tt.err, err)
		}
	}
}
