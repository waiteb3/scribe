// Copyright (C) Brandon Waite 2020  - All Rights Reserved
// Unauthorized copying of this file, via any medium, is strictly prohibited
// Proprietary
// Updated by Brandon Waite, May 28 2020

package main

import (
	"context"
	"database/sql"
	"encoding/base64"
	"fmt"
	"os"
	"strconv"
	"strings"
)

func RecordCommandV1(ctx context.Context, record string, timestamp string) error {
	file := ctx.Value("file").(*os.File)

	logline := fmt.Sprintf("%s:%s\n", timestamp, base64.StdEncoding.EncodeToString([]byte(record)))
	if _, err := file.WriteString(logline); err != nil {
		return err
	}

	return nil
}

func IndexCommandV1(ctx context.Context, record string, timestamp string) error {
	db := ctx.Value("db").(*sql.DB)

	if _, err := db.Exec("INSERT INTO history(command, timestamp) VALUES (?, ?)", record, timestamp); err != nil {
		return err
	}

	return nil
}

// sqlite replacement:
// a cache last 10 commands for each
func Record(ctx context.Context, fullCmd string) error {
	timestamp := strconv.Itoa(int(now()))
	// TODO session specific log search as well
	if err := initDebug(); err != nil {
		return err
	}
	defer debug.Close()

	// TODO not if histfile
	//	if os.Getenv("HISTFILE") == "" {
	//		debug.WriteString("HISTFILE unset, not recording " + time.Now().String() + "\n")
	//        debug.WriteString(strings.Join(os.Environ(), ";") + "\n")
	//		return nil
	//	} else {
	//        debug.WriteString("HISTFILE: " + os.Getenv("HISTFILE") + "\n")
	//    }

	cmd := strings.SplitN(fullCmd, " ", 2)[0]
	if cmd == "scribe" {
		return nil
	}

	// TODO after 1k lines rotate file
	logfile, err := dir.Filepath(history, LATEST)
	if err != nil {
		return err
	}

	file, err := os.OpenFile(logfile, os.O_APPEND|os.O_WRONLY, 0644)
	if os.IsNotExist(err) {
		file, err = os.OpenFile(logfile, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0644)
		if err != nil {
			return err
		}
		if _, err := file.WriteString("version=1,encoder=base64\n---\n"); err != nil {
			return err
		}
	}

	if err != nil {
		return err
	}
	defer file.Close()

	dbpath, err := dir.Filepath(data, indexDatabase)
	if err != nil {
		return err
	}

	db, err := sql.Open("sqlite3", dbpath)
	if err != nil {
		return err
	}
	defer db.Close()

	if err := RecordCommandV1(context.WithValue(ctx, "file", file), fullCmd, timestamp); err != nil {
		return err
	}

	if err := IndexCommandV1(context.WithValue(ctx, "db", db), fullCmd, timestamp); err != nil {
		return err
	}

	return nil
}
