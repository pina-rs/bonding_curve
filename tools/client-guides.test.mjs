// The published clients are 0.2 and depend on Pina 0.24. Cargo's "0.1"
// requirement and Dart's ^0.1.0 both exclude 0.2, so a guide that still
// says 0.1 does not install this release. The same guides have to name
// every event the program emits, or an indexer drops creator handovers
// and dust sweeps.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

/**
 * @param {string} path
 * @returns {string}
 */
function read(path) {
	return readFileSync(path, "utf8");
}

/**
 * @param {string} text
 * @param {RegExp} pattern
 * @param {string} label
 * @returns {string}
 */
function capture(text, pattern, label) {
	const match = pattern.exec(text);
	assert.ok(match?.[1], `${label} did not match ${pattern}`);
	return match[1];
}

test("client guides require the published package versions", () => {
	const rust = capture(
		read("clients/rust/pina_bonding_curve/Cargo.toml"),
		/^version = "(\d+\.\d+)\.\d+"/m,
		"rust client",
	);
	const cpi = capture(
		read("clients/cpi/pina_bonding_curve/Cargo.toml"),
		/^version = "(\d+\.\d+)\.\d+"/m,
		"cpi client",
	);
	const dart = capture(
		read("clients/dart/pubspec.yaml"),
		/^version: (\d+\.\d+\.\d+)/m,
		"dart client",
	);
	const pina = capture(
		read("Cargo.toml"),
		/pina = \{[^}]*version = "=(\d+\.\d+)\.\d+"/m,
		"pina",
	);
	const solanaInstruction = capture(
		read("Cargo.toml"),
		/solana-instruction = \{[^}]*version = "~(\d+\.\d+)"/m,
		"solana-instruction",
	);

	/** @type {Array<[string, string]>} */
	const expected = [
		["docs/rust-client.md", `pina_bonding_curve_client = "${rust}"`],
		["docs/rust-client.md", `solana-instruction = "${solanaInstruction}"`],
		[
			"clients/rust/pina_bonding_curve/README.md",
			`pina_bonding_curve_client = "${rust}"`,
		],
		["docs/cpi.md", `pina_bonding_curve_cpi = "${cpi}"`],
		["docs/cpi.md", `version = "${pina}"`],
		[
			"clients/cpi/pina_bonding_curve/README.md",
			`pina_bonding_curve_cpi = "${cpi}"`,
		],
		["docs/dart.md", `pina_bonding_curve: ^${dart}`],
		["clients/dart/README.md", `pina_bonding_curve: ^${dart}`],
	];
	for (const [file, needle] of expected) {
		assert.ok(
			read(file).includes(needle),
			`${file} must contain ${
				JSON.stringify(needle)
			} so it resolves to the published release`,
		);
	}
});

test("client guides name every emitted event", () => {
	const instructions = read("docs/instructions.md");
	const events = capture(
		instructions,
		/## Events\n\n([\s\S]*?)\n## Errors/,
		"events section",
	);
	const names = [...events.matchAll(/^\| \d+ \| `([A-Za-z]+)` /gm)].map(
		(match) => match[1],
	);
	assert.deepEqual(names, [
		"ConfigCreated",
		"LaunchCreated",
		"Traded",
		"Completed",
		"Graduated",
		"Claimed",
		"LaunchCreatorChanged",
		"DustSwept",
	]);

	for (const file of ["docs/rust-client.md", "docs/integrating.md"]) {
		const text = read(file);
		for (const name of names) {
			assert.ok(text.includes(name), `${file} omits ${name}`);
		}
	}

	const dart = read("docs/dart.md");
	for (const name of names) {
		assert.ok(dart.includes(`${name}Event`), `docs/dart.md omits ${name}Event`);
	}

	const typescript = read("docs/typescript.md");
	for (const name of names) {
		const camel = name.charAt(0).toLowerCase() + name.slice(1);
		assert.ok(
			typescript.includes(`\`${camel}\``),
			`docs/typescript.md omits ${camel}`,
		);
	}
});
