import { fileURLToPath } from "node:url";
import type { RepoDocsConfig } from "./repo-docs";

/** Facts about the project that the landing page and docs share. */
export const site = {
	name: "Pina Bonding Curve",
	/** The wordmark: the family name, then the product in the accent colour. */
	brand: { family: "Pina", product: "Bonding Curve" },
	tagline: "Self-serve token launchpads for Solana",
	url: "https://bonding-curve.pina.rs",
	description:
		"Self-serve token launchpads for Solana, built with Pina. Launches sell along a curve you design and graduate into a Pina AMM pool at exactly the price they reached.",
	repository: "https://github.com/pina-rs/bonding_curve",
	branch: "main",
	programId: "CurveqeE6jzkyHQMcWaGENzd7jrd8m9R1u4dZ17GnSa9",
	pina: "https://github.com/pina-rs/pina",
	/** The other Pina product, linked from the footer. */
	sibling: { name: "Pina AMM", href: "https://amm.pina.rs" },
} as const;

/** The published packages, in the order the landing page lists them. */
export const packages = [
	{
		name: "@pina-rs/bonding-curve",
		registry: "npm",
		href: "https://www.npmjs.com/package/@pina-rs/bonding-curve",
		summary: "TypeScript client for @solana/kit",
	},
	{
		name: "pina_bonding_curve_client",
		registry: "crates.io",
		href: "https://crates.io/crates/pina_bonding_curve_client",
		summary: "Rust client: builders, decoders, PDAs, events",
	},
	{
		name: "pina_bonding_curve_cpi",
		registry: "crates.io",
		href: "https://crates.io/crates/pina_bonding_curve_cpi",
		summary: "no_std client for cross-program calls",
	},
	{
		name: "pina_bonding_curve",
		registry: "pub.dev",
		href: "https://pub.dev/packages/pina_bonding_curve",
		summary: "Dart and Flutter client for solana_kit",
	},
	{
		name: "pina_bonding_curve_cli",
		registry: "crates.io",
		href: "https://crates.io/crates/pina_bonding_curve_cli",
		summary: "The pina-curve tool: design curves, run launches",
	},
] as const;

/** Where the guides live relative to the site, and how they are served. */
const docsSource = { directory: "../docs/", route: "docs" } as const;

/** Resolves the guide configuration against the site's root directory. */
export function resolveRepoDocsConfig(siteRoot: URL): RepoDocsConfig {
	return {
		repoRoot: fileURLToPath(new URL("../", siteRoot)),
		docsDir: fileURLToPath(new URL(docsSource.directory, siteRoot)),
		route: docsSource.route,
		repoUrl: site.repository,
		branch: site.branch,
	};
}
