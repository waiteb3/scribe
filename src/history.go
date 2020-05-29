// Copyright (C) Brandon Waite 2020  - All Rights Reserved
// Unauthorized copying of this file, via any medium, is strictly prohibited
// Proprietary
// Updated by Brandon Waite, May 28 2020

package main

import (
	"bufio"
	"context"
	"database/sql"
	"encoding/base64"
	"errors"
	"fmt"
	"io/ioutil"
	"os"
	"path"
	"strconv"
	"strings"
)

func MigrateHistory(ctx context.Context) error {
	historyFolder, err := dir.Folder(history)
	if err != nil {
		return err
	}

	historyFiles, err := ioutil.ReadDir(historyFolder)
	if err != nil {
		return err
	}

	for _, file := range historyFiles {
		if !strings.HasPrefix(file.Name(), "log") {
			continue
		}

		if err := MigrateHistoryFileV0toV1(ctx, path.Join(historyFolder, file.Name())); err != nil {
			return err
		}
	}

	latest, err := dir.Filepath(history, LATEST)
	if err != nil {
		return err
	}

	if err := MigrateHistoryFileV0toV1(ctx, latest); err != nil {
		return err
	}

	return nil
}

func MigrateHistoryFileV0toV1(ctx context.Context, filename string) error {
	v0, err := os.Open(filename)
	if err != nil {
		return err
	}
	defer v0.Close()

	scanner := bufio.NewScanner(v0)
	timestamp := ""
	fullCmd := ""

	if !scanner.Scan() {
		// meta
		return errors.New("TODO1")
	}
	if !scanner.Scan() {
		// splitter
		return errors.New("TODO2")
	}
	if !scanner.Scan() {
		// first line
		return errors.New("TODO3")
	}

	parts := strings.SplitN(scanner.Text(), ":", 2)
	timestamp, fullCmd = parts[0], parts[1]

	cmds := make([]string, 0, 1000)
	timestamps := make([]string, 0, 1000)

	for scanner.Scan() {
		parts := strings.SplitN(scanner.Text(), ":", 2)
		maybeTs := parts[0]
		if _, err := strconv.Atoi(maybeTs); err == nil && len(maybeTs) == 10 && len(parts) == 2 {
			cmds = append(cmds, fullCmd)
			timestamps = append(timestamps, timestamp)

			timestamp, fullCmd = parts[0], parts[1]
		} else {
			fullCmd += "\n" + scanner.Text()
		}
	}

	v0.Close()

	v1, err := os.OpenFile(filename, os.O_CREATE|os.O_WRONLY, 0644)
	if err != nil {
		return err
	}
	defer v1.Close()

	if _, err := v1.WriteString("version=1,encoding=base64\n---\n"); err != nil {
		return err
	}

	for i, cmd := range cmds {
		timestamp := timestamps[i]
		if err := RecordCommandV1(context.WithValue(ctx, "file", v1), cmd, timestamp); err != nil {
			return err
		}
	}

	return nil
}

func ImportHistoryFile(ctx context.Context, db *sql.DB, filename string) error {
	file, err := os.Open(filename)
	if err != nil {
		return err
	}
	defer file.Close()

	scanner := bufio.NewScanner(file)
	if !scanner.Scan() {
		// meta
		return errors.New("TODO1")
	}
	if !scanner.Scan() {
		// splitter
		return errors.New("TODO2")
	}

	line := 3
	for scanner.Scan() {
		parts := strings.SplitN(scanner.Text(), ":", 2)
		if len(parts) != 2 {
			return fmt.Errorf("Malformed line in file %s at line %d", filename, line)
		}

		timestamp, encoded := parts[0], parts[1]
		command, err := base64.StdEncoding.DecodeString(encoded)
		if err != nil {
			return fmt.Errorf("Malformed line in file %s at line %d: %s", filename, line, err.Error())
		}

		if err := IndexCommandV1(context.WithValue(ctx, "db", db), string(command), timestamp); err != nil {
			return err
		}

		line++
	}

	if err := scanner.Err(); err != nil {
		return err
	}

	return nil
}

type Writer func(ctx context.Context, record string, timestamp string) error

func ResetFromHistory(ctx context.Context) error {
	if err := initDebug(); err != nil {
		return err
	}
	defer debug.Close()

	dbpath, err := dir.Filepath(data, indexDatabase)
	if err != nil {
		return err
	}

	db, err := sql.Open("sqlite3", dbpath)
	if err != nil {
		return err
	}
	defer db.Close()

	// TODO #002# probably less than ideal, should do a lockfile or something sensible
	if _, err := db.ExecContext(ctx, "DELETE FROM history"); err != nil {
		return err
	}

	historyFolder, _ := dir.Folder(history)
	historyFiles, _ := ioutil.ReadDir(historyFolder)
	for _, file := range historyFiles {
		if !strings.HasPrefix(file.Name(), "log.") {
			continue
		}

		if err := ImportHistoryFile(
			ctx,
			db,
			path.Join(historyFolder, file.Name()),
		); err != nil {
			return err
		}
	}

	latest, _ := dir.Filepath(history, LATEST)
	if err := ImportHistoryFile(ctx, db, latest); err != nil {
		return err
	}

	return nil
}
