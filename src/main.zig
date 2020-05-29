// Copyright (C) Brandon Waite 2020  - All Rights Reserved
// Unauthorized copying of this file, via any medium, is strictly prohibited
// Proprietary
// Updated by Brandon Waite, May 28 2020

const std = @import("std");
const sqlite = @import("./sqlite.zig");

const stdout = &std.io.getStdOut().outStream().stream;

fn init() !void {
    const zshInit = @embedFile("init.zsh");
    try stdout.print("{}\n", .{zshInit});
}

fn record(cmdargs: [][]u8) !void {
    try stdout.print("recorded", .{});
    for (cmdargs) |arg| {
        try stdout.print(" {}", .{arg});
    }
    try stdout.print("\n", .{});
}

pub fn main() anyerror!void {
    const db = try sqlite.Database.new("test.db");
    defer db.close();
    _ = try db.execute("CREATE TABLE test_table (id INT PRIMARY KEY, value TEXT NOT NULL)");
    _ = try db.execute("INSERT INTO test_table VALUES (1, 'TEST'), (2, 'other')");

    const args = try std.process.argsAlloc(std.heap.page_allocator);
    defer std.process.argsFree(std.heap.page_allocator, args);

    if (args.len < 2) {
        try stdout.print("Help TODO. try `scribe init` \n", .{});
        return;
    }

    const subcmd = args[1];
    const cmdargs = args[2..];

    if (std.mem.eql(u8, subcmd, "init")) {
        try init();
    } else if (std.mem.eql(u8, subcmd, "record")) {
        try record(cmdargs);
    } else {
        std.debug.warn("Unknown option: {}", .{subcmd});
    }
}
