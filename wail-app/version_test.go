package main

import (
	"os"
	"regexp"
	"strings"
	"testing"
)

func TestAppVersionMatchesCargoToml(t *testing.T) {
	data, err := os.ReadFile("../Cargo.toml")
	if err != nil {
		t.Fatalf("read Cargo.toml: %v", err)
	}
	inSection := false
	re := regexp.MustCompile(`^version\s*=\s*"([^"]+)"`)
	var want string
	for _, line := range strings.Split(string(data), "\n") {
		trimmed := strings.TrimSpace(line)
		if strings.HasPrefix(trimmed, "[") {
			inSection = trimmed == "[workspace.package]"
			continue
		}
		if inSection {
			if m := re.FindStringSubmatch(trimmed); m != nil {
				want = m[1]
				break
			}
		}
	}
	if want == "" {
		t.Fatal("could not find workspace.package version in Cargo.toml")
	}
	if appVersion != want {
		t.Fatalf("appVersion = %q, Cargo.toml workspace version = %q — run the release workflow or update wail-app/version.go", appVersion, want)
	}
}
