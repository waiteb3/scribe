// Copyright (C) Brandon Waite 2020  - All Rights Reserved
// Unauthorized copying of this file, via any medium, is strictly prohibited
// Proprietary
// Updated by Brandon Waite, May 28 2020

package main

import (
	"errors"
	"fmt"
	"os"
	"path/filepath"
)

type StorageDirectory struct {
	root  string
	tails map[Tail]string
}

var ErrorTailMissing = errors.New("Unable to find Tail")

func (sd *StorageDirectory) Filepath(tail Tail, filename string) (string, error) {
	path := sd.tails[tail]
	if path == "" {
		fmt.Println(sd.tails)
		return "", fmt.Errorf("%s: %s", ErrorTailMissing, string(tail))
	}

	return filepath.Join(path, filename), nil
}

func (sd *StorageDirectory) Folder(tail Tail) (string, error) {
	path := sd.tails[tail]
	if path == "" {
		fmt.Println(sd.tails)
		return "", fmt.Errorf("%s: %s", ErrorTailMissing, string(tail))
	}

	return path, nil
}

func (sd *StorageDirectory) Ensure() (bool, error) {
	var alreadyExists bool
	if _, err := os.Stat(sd.root); os.IsNotExist(err) {
		alreadyExists = false
	} else {
		alreadyExists = true
	}

	for _, fullpath := range sd.tails {
		if err := os.MkdirAll(fullpath, 0755); err != nil && !os.IsExist(err) {
			return alreadyExists, err
		}
	}
	return alreadyExists, nil
}

func (sd *StorageDirectory) resgister(t Tail, fullpath string) error {
	if sd.tails[t] != "" {
		return errors.New("Duplicate tail: '" + string(t) + "' already exists")
	}
	sd.tails[t] = fullpath
	return nil
}

// stupid lack of generics stupid
type Node interface {
	build(*StorageDirectory, ...string) error
}

type Tail string

func (t Tail) build(sd *StorageDirectory, paths ...string) error {
	fullpath := filepath.Join(paths...)

	return sd.resgister(t, fullpath)
}

type Directory map[string]Node

func (d Directory) build(sd *StorageDirectory, paths ...string) error {
	for name, node := range d {
		if err := node.build(sd, append(paths, name)...); err != nil {
			return err
		}
	}

	return nil
}

const (
	Relative int = iota
	Home
)

func NewStorageDirectory(root string, mode int, structure Node) (*StorageDirectory, error) {
	if mode == Home {
		home, err := os.UserHomeDir()
		if err != nil {
			return nil, err
		}

		root = filepath.Join(home, root)
	}

	sd := &StorageDirectory{
		root:  root,
		tails: map[Tail]string{},
	}

	if err := structure.build(sd, root); err != nil {
		return nil, err
	}
	return sd, nil
}
