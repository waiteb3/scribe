// Copyright (C) Brandon Waite 2020  - All Rights Reserved
// Unauthorized copying of this file, via any medium, is strictly prohibited
// Proprietary
// Updated by Brandon Waite, May 28 2020

package main

import (
	"context"
	"fmt"
	"os"
	"time"
)

func now() int64 {
	return time.Now().Unix()
}

const indexDatabase = "index.db"

var dir *StorageDirectory
var data = Tail("data")
var history = Tail("history")
var LATEST = "LATEST"

func main() {
	if len(os.Args) < 2 {
		fmt.Fprintf(os.Stderr, "Help TODO\n")
		os.Exit(1)
	}

	var err error
	dir, err = NewStorageDirectory(".scribe", Home, Directory(map[string]Node{
		"data":    data,
		"history": history,
	}))
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}

	cmd, args := os.Args[1], os.Args[2:]

	ctx := context.TODO()

	switch cmd {
	case "init":
		err = Init(ctx)
	case "record":
		err = Record(ctx, args[0])
	case "search":
		err = Search(ctx, args)
	case "reset-index":
		err = ResetFromHistory(ctx)
	case "migrate-history":
		err = MigrateHistory(ctx)
	default:
		err = fmt.Errorf("Unknown option: '%s'\n", cmd)
	}

	if err != nil {
		fmt.Fprintln(os.Stderr, err)
	}
}
