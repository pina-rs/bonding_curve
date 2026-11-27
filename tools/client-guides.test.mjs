// The client guides must point at the current published package versions and
// stay synchronized with monochange's release automation so generated release
// PRs update every install snippet automatically. The same guides also have to
// name every event the program emits, or an indexer drops creator handovers
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

/**
 * Parse regex-based `versioned_files` entries from `monochange.toml`.
 *
 * @param {string} configText
 * @returns {Array<{ path: string; regex: string }>}
 */
function parseRegexVersionedFiles(configText) {
	const block = capture(
		configText,
		/^\[group\.bonding_curve\][\s\S]*?^versioned_files = \[([\s\S]*?)^\]/m,
		"bonding_curve versioned_files",
	);
	const entryPattern =
		/\{\s*path\s*=\s*"([^"]+)"\s*,\s*regex\s*=\s*(?:'([^']*)'|"((?:\\.|[^"\\])*)")\s*\}/g;
	/** @type {Array<{ path: string; regex: string }>} */
	const entries = [];
	for (const match of block.matchAll(entryPattern)) {
		const path = match[1];
		const regex = match[2] ?? match[3].replace(/\\(.)/g, "$1");
		entries.push({ path, regex });
	}
	return entries;
}

test("client guides require the published package versions", () => {
	const rust = capture(
		read("clients/rust/pina_bonding_curve/Cargo.toml"),
		/^version = "(\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?)"/m,
		"rust client",
	);
	const cpi = capture(
		read("clients/cpi/pina_bonding_curve/Cargo.toml"),
		/^version = "(\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?)"/m,
		"cpi client",
	);
	const dart = capture(
		read("clients/dart/pubspec.yaml"),
		/^version: (\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?)/m,
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

	/** @type {Array<[string, string, string]>} */
	const releasedGuides = [
		["docs/rust-client.md", `pina_bonding_curve_client = "${rust}"`, rust],
		[
			"clients/rust/pina_bonding_curve/README.md",
			`pina_bonding_curve_client = "${rust}"`,
			rust,
		],
		["docs/cpi.md", `pina_bonding_curve_cpi = "${cpi}"`, cpi],
		[
			"clients/cpi/pina_bonding_curve/README.md",
			`pina_bonding_curve_cpi = "${cpi}"`,
			cpi,
		],
		["docs/dart.md", `pina_bonding_curve: ^${dart}`, dart],
		["clients/dart/README.md", `pina_bonding_curve: ^${dart}`, dart],
	];

	/** @type {Array<[string, string]>} */
	const dependencyGuides = [
		["docs/rust-client.md", `solana-instruction = "${solanaInstruction}"`],
		["docs/cpi.md", `version = "${pina}"`],
	];

	/** @type {Array<[string, string]>} */
	const expected = [
		...releasedGuides.map((
			[file, needle],
		) => /** @type {[string, string]} */ ([file, needle])),
		...dependencyGuides,
	];
	for (const [file, needle] of expected) {
		assert.ok(
			read(file).includes(needle),
			`${file} must contain ${
				JSON.stringify(needle)
			} so it resolves to the published release`,
		);
	}

	const versionedFiles = parseRegexVersionedFiles(read("monochange.toml"));
	const coveredFiles = new Set(versionedFiles.map((entry) => entry.path));
	for (const [file] of expected) {
		assert.ok(
			coveredFiles.has(file),
			`monochange.toml must declare a versioned_files entry for ${file}`,
		);
	}

	for (const [file, needle, version] of releasedGuides) {
		const entries = versionedFiles.filter((entry) => entry.path === file);
		assert.ok(
			entries.length > 0,
			`monochange.toml must declare a regex versioned_files entry for ${file}`,
		);
		const matched = entries.some((entry) => {
			const pattern = new RegExp(entry.regex);
			return (
				pattern.exec(needle)?.groups?.version === version &&
				pattern.exec(read(file))?.groups?.version === version
			);
		});
		assert.ok(
			matched,
			`monochange.toml regex for ${file} must match ${
				JSON.stringify(needle)
			} and capture version ${version}`,
		);
	}

	for (const [file, needle] of dependencyGuides) {
		const entries = versionedFiles.filter((entry) => entry.path === file);
		for (const entry of entries) {
			assert.equal(
				new RegExp(entry.regex).exec(needle),
				null,
				`monochange.toml regex for ${file} must not rewrite external dependency ${
					JSON.stringify(needle)
				}`,
			);
		}
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
