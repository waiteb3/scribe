// Copyright (C) Brandon Waite 2020  - All Rights Reserved
// Unauthorized copying of this file, via any medium, is strictly prohibited
// Proprietary
// Updated by Brandon Waite, May 28 2020

package main

import "os"

var debug *os.File

func initDebug() error {
	debugFilename, err := dir.Filepath(data, "debug.log")
	if err != nil {
		return err
	}

	debug, err = os.OpenFile(debugFilename, os.O_CREATE|os.O_APPEND|os.O_WRONLY, 0644)
	if err != nil {
		return err
	}

	return nil
}
