import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

import { describe, expect, it } from "vitest";

// Fails on invisible characters that break builds or diffs, e.g. a `\b` that
// became a backspace when a Windows path went through a scripted replace.
const repoRoot = resolve(import.meta.dirname, "../..");

const TAB = 0x09;
const LF = 0x0a;

function git(args: string[], input?: string): string {
  return execFileSync("git", args, { cwd: repoRoot, encoding: "utf8", input });
}

/** Tracked files plus untracked files that are not ignored. */
function repoFiles(): string[] {
  return git(["ls-files", "-z", "--cached", "--others", "--exclude-standard"])
    .split("\0")
    .filter((path) => path !== "");
}

/** Files marked `binary` in .gitattributes. */
function binaryFiles(paths: string[]): Set<string> {
  const output = git(["check-attr", "-z", "--stdin", "binary"], paths.join("\0"));
  const fields = output.split("\0");
  const binary = new Set<string>();
  for (let i = 0; i + 2 < fields.length; i += 3) {
    const path = fields[i];
    if (path !== undefined && fields[i + 2] === "set") {
      binary.add(path);
    }
  }
  return binary;
}

function findProblems(path: string): string[] {
  const bytes = readFileSync(resolve(repoRoot, path));
  const problems: string[] = [];
  if (bytes[0] === 0xef && bytes[1] === 0xbb && bytes[2] === 0xbf) {
    problems.push(`${path}: starts with a UTF-8 BOM`);
  }
  let line = 1;
  let column = 1;
  for (const byte of bytes) {
    if (byte === LF) {
      line += 1;
      column = 1;
      continue;
    }
    const isControl = (byte < 0x20 && byte !== TAB) || byte === 0x7f;
    if (isControl) {
      const hex = byte.toString(16).padStart(2, "0");
      problems.push(`${path}:${String(line)}:${String(column)}: control character 0x${hex}`);
    }
    column += 1;
  }
  return problems;
}

describe("repo hygiene", () => {
  it("has no control characters, CRs or BOMs in text files", () => {
    const paths = repoFiles();
    const binary = binaryFiles(paths);
    const problems = paths.filter((path) => !binary.has(path)).flatMap(findProblems);
    expect(problems).toEqual([]);
  });
});
