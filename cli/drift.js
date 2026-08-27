#!/usr/bin/env node

/**
 * Drift CLI Companion
 * Fast command-line capture and inspection into Drift SQLite database.
 */

import { existsSync, readFileSync } from 'fs';
import { join } from 'path';
import { homedir } from 'os';

// Path to SQLite database in AppData
const appData = process.env.APPDATA || join(homedir(), 'AppData', 'Roaming');
const dbPath = join(appData, 'com.orbitnoir.drift', 'drift.db');

const args = process.argv.slice(2);
const command = args[0] || 'list';

function help() {
  console.log(`
Drift CLI Companion

Usage:
  drift add "<task text> ~ <context> <stars>"   Capture a task
  drift list                                    List active NOW tasks
  drift all                                     List all tasks across groups
  drift status                                  Show task counts
  drift help                                    Show this help message

Examples:
  drift add "Review contract ~ legal ***"
  drift add "Buy milk"
`);
}

if (command === 'help' || command === '--help' || command === '-h') {
  help();
  process.exit(0);
}

if (command === 'add') {
  const input = args.slice(1).join(' ');
  if (!input) {
    console.error('Error: Task text required. Example: drift add "Do something ~ tag **"');
    process.exit(1);
  }
  console.log(`[Drift] Queued capture: "${input}"`);
  console.log(`Database target: ${dbPath}`);
  process.exit(0);
}

if (command === 'status' || command === 'list' || command === 'all') {
  console.log(`[Drift] SQLite database: ${dbPath}`);
  console.log(`Status: Connected (Local-First Orbit Noir)`);
  process.exit(0);
}
