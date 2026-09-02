#!/usr/bin/env node

/**
 * Fail-closed check for Bubble Tea's sibling Cargo source graph.
 *
 * The consuming root owns source unification. Every audited sibling must
 * resolve exactly once from its adjacent checkout; registry or duplicate
 * substitutions invalidate the reviewed build input.
 */

import { execFileSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const siblingRoot = path.dirname(repositoryRoot);
const siblingNames = [
  "rusty-bubbles",
  "rusty-colorprofile",
  "rusty-lipgloss",
  "rusty-testkit",
  "rusty-ultraviolet",
  "rusty-x-ansi",
];

/** Validate that each audited package has one adjacent path source. */
function assertSiblingSources(metadata, root = siblingRoot) {
  for (const name of siblingNames) {
    const matches = metadata.packages.filter((entry) => entry.name === name);
    if (matches.length !== 1) {
      throw new Error(`expected exactly one ${name} package, found ${matches.length}`);
    }

    const expectedManifest = path.resolve(root, name, "Cargo.toml");
    const actualManifest = path.resolve(matches[0].manifest_path);
    if (matches[0].source !== null || actualManifest !== expectedManifest) {
      throw new Error(
        `${name} resolved from ${matches[0].manifest_path} (${matches[0].source}), ` +
          `expected adjacent path ${expectedManifest}`,
      );
    }
  }
}

// Negative regression: a registry substitution must be rejected before the
// live graph is inspected.
const invalidMetadata = {
  packages: siblingNames.map((name) => ({
    name,
    manifest_path: path.resolve(siblingRoot, name, "Cargo.toml"),
    source: name === "rusty-x-ansi" ? "registry+https://github.com/rust-lang/crates.io-index" : null,
  })),
};
let rejectedInvalidSource = false;
try {
  assertSiblingSources(invalidMetadata);
} catch {
  rejectedInvalidSource = true;
}
if (!rejectedInvalidSource) {
  throw new Error("dependency-source guard accepted an injected registry substitution");
}

const metadata = JSON.parse(
  execFileSync(
    "cargo",
    [
      "+1.98.0",
      "metadata",
      "--locked",
      "--manifest-path",
      path.join(repositoryRoot, "Cargo.toml"),
      "--format-version",
      "1",
    ],
    { encoding: "utf8" },
  ),
);
assertSiblingSources(metadata);
console.log("all audited Bubble Tea sibling dependencies resolve from one adjacent path source");
