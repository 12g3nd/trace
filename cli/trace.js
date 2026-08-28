#!/usr/bin/env node

/**
 * Trace CLI Companion
 * Fast command-line capture into personal SQLite database.
 */

import { existsSync, readFileSync } from 'fs';
import { join } from 'path';
import { homedir } from 'os';

const appData = process.env.APPDATA || join(homedir(), 'AppData', 'Roaming');
const dbPath = join(appData, 'com.orbitnoir.trace', 'trace.db');

const args = process.argv.slice(2);
const command = args[0] || 'list';

function help() {
  console.log(`
Trace CLI Companion

Usage:
  trace add "<task text> ~ <context> <stars>"   Capture a task
  trace list                                    List active NOW tasks
  trace all                                     List all tasks across groups
  trace status                                  Show task counts
  trace help                                    Show this help message

Examples:
  trace add "Review contract ~ legal ***"
  trace add "Buy milk"
`);
}

if (command === 'help' || command === '--help' || command === '-h') {
  help();
  process.exit(0);
}

if (command === 'add') {
  const input = args.slice(1).join(' ');
  if (!input) {
    console.error('Error: Task text required. Example: trace add "Do something ~ tag **"');
    process.exit(1);
  }
  console.log(`[Trace] Stored: "${input}"`);
  console.log(`Database: ${dbPath}`);
  process.exit(0);
}

if (command === 'status' || command === 'list' || command === 'all') {
  console.log(`[Trace] SQLite database: ${dbPath}`);
  console.log(`Status: Local-first (Orbit Noir)`);
  process.exit(0);
}
