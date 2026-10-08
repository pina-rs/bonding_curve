/**
 * The model behind the landing page's launch simulator: the reference launch
 * from `docs/curves.md` (one billion 6-decimal tokens raising 85 SOL), priced
 * with the program's segment formulas. It works in floating point because it
 * only draws and describes; the program's integer math additionally rounds
 * every step in the curve's favour.
 *
 * Prices are lamports per base unit, as on chain. A square-root price is the
 * program's Q64.64 value divided by 2^64.
 */

const Q64 = 2 ** 64;
const BASE_UNITS_PER_TOKEN = 1e6;
const LAMPORTS_PER_SOL = 1e9;

export interface Segment {
	/** The segment's upper square-root price. */
	upperSqrtPrice: number;
	liquidity: number;
}

export interface Curve {
	startSqrtPrice: number;
	segments: readonly Segment[];
}

/** The reference curve: one segment, exactly as `docs/curves.md` configures it. */
export const ONE_SEGMENT: Curve = {
	startSqrtPrice: 97_610_000_000_000_000 / Q64,
	segments: [{
		upperSqrtPrice: 780_880_000_000_000_000 / Q64,
		liquidity: 5_000_000_000_000,
	}],
};

/**
 * Three segments from the same start: a shallow first segment moves the price
 * quickly for early buyers, and a deep last one keeps it steady while the
 * launch fills. It raises the same 85 SOL.
 */
export const THREE_SEGMENTS: Curve = {
	startSqrtPrice: ONE_SEGMENT.startSqrtPrice,
	segments: [
		{
			upperSqrtPrice: ONE_SEGMENT.startSqrtPrice * 2,
			liquidity: 1_900_000_000_000,
		},
		{
			upperSqrtPrice: ONE_SEGMENT.startSqrtPrice * 3.2,
			liquidity: 4_700_000_000_000,
		},
		{
			upperSqrtPrice: ONE_SEGMENT.startSqrtPrice * 8,
			liquidity: 9_000_000_000_000,
		},
	],
};

/** The reference launchpad's terms from `docs/launchpads.md`. */
export const TERMS = {
	migrationQuote: 85 * LAMPORTS_PER_SOL,
	totalSupply: 1_000_000_000 * BASE_UNITS_PER_TOKEN,
	creatorAllocation: 50_000_000 * BASE_UNITS_PER_TOKEN,
	startFeeRate: 0.5,
	endFeeRate: 0.01,
	feeDecaySeconds: 60,
	migrationFeeRate: 0.02,
} as const;

export interface CurvePoint {
	sqrtPrice: number;
	/** Base units sold since the start. */
	baseSold: number;
	/** Lamports raised since the start, excluding fees. */
	quoteRaised: number;
}

/** Walks up the curve from its start, spending `quote` lamports. */
export function pointAtQuote(curve: Curve, quote: number): CurvePoint {
	let sqrtPrice = curve.startSqrtPrice;
	let remaining = quote;
	let baseSold = 0;
	for (const segment of curve.segments) {
		if (sqrtPrice >= segment.upperSqrtPrice) {
			continue;
		}
		const toCross = segment.liquidity * (segment.upperSqrtPrice - sqrtPrice);
		if (remaining <= toCross) {
			const next = sqrtPrice + remaining / segment.liquidity;
			baseSold += segment.liquidity * (1 / sqrtPrice - 1 / next);
			return { sqrtPrice: next, baseSold, quoteRaised: quote };
		}
		baseSold += segment.liquidity *
			(1 / sqrtPrice - 1 / segment.upperSqrtPrice);
		remaining -= toCross;
		sqrtPrice = segment.upperSqrtPrice;
	}
	return { sqrtPrice, baseSold, quoteRaised: quote - remaining };
}

/** Walks up the curve from its start until `base` units are sold. */
export function pointAtBase(curve: Curve, base: number): CurvePoint {
	let sqrtPrice = curve.startSqrtPrice;
	let remaining = base;
	let quoteRaised = 0;
	for (const segment of curve.segments) {
		if (sqrtPrice >= segment.upperSqrtPrice) {
			continue;
		}
		const supply = segment.liquidity *
			(1 / sqrtPrice - 1 / segment.upperSqrtPrice);
		if (remaining <= supply) {
			const next = 1 / (1 / sqrtPrice - remaining / segment.liquidity);
			quoteRaised += segment.liquidity * (next - sqrtPrice);
			return { sqrtPrice: next, baseSold: base, quoteRaised };
		}
		quoteRaised += segment.liquidity * (segment.upperSqrtPrice - sqrtPrice);
		remaining -= supply;
		sqrtPrice = segment.upperSqrtPrice;
	}
	return { sqrtPrice, baseSold: base - remaining, quoteRaised };
}

/** Where a launch completes: the curve has raised the migration threshold. */
export function migrationPoint(curve: Curve): CurvePoint {
	return pointAtQuote(curve, TERMS.migrationQuote);
}

/** Base units sold by the end of each segment, for marking segment boundaries. */
export function segmentEnds(curve: Curve): number[] {
	let sqrtPrice = curve.startSqrtPrice;
	let baseSold = 0;
	return curve.segments.map((segment) => {
		baseSold += segment.liquidity *
			(1 / sqrtPrice - 1 / segment.upperSqrtPrice);
		sqrtPrice = segment.upperSqrtPrice;
		return baseSold;
	});
}

export function price(sqrtPrice: number): number {
	return sqrtPrice * sqrtPrice;
}

/** A price in lamports per base unit, as SOL per million whole tokens. */
export function solPerMillionTokens(lamportsPerBaseUnit: number): number {
	return (lamportsPerBaseUnit * BASE_UNITS_PER_TOKEN * 1e6) / LAMPORTS_PER_SOL;
}

export function tokens(baseUnits: number): number {
	return baseUnits / BASE_UNITS_PER_TOKEN;
}

export function sol(lamports: number): number {
	return lamports / LAMPORTS_PER_SOL;
}

/** The trading fee rate `seconds` after the launch opens: a straight line from the opening rate. */
export function feeRateAt(seconds: number): number {
	if (seconds >= TERMS.feeDecaySeconds) {
		return TERMS.endFeeRate;
	}
	const elapsed = Math.max(seconds, 0);
	return TERMS.startFeeRate -
		((TERMS.startFeeRate - TERMS.endFeeRate) * elapsed) / TERMS.feeDecaySeconds;
}

export interface BuyQuote {
	/** Lamports the buyer pays, fee included. */
	paid: number;
	fee: number;
	/** Base units the buyer receives. */
	base: number;
	/** Whether the buy reaches the migration price and completes the launch. */
	completes: boolean;
}

/**
 * A buy of `quoteIn` lamports (fee included) from `from`. A buy that would
 * pass the migration price stops there, and the buyer pays only for what the
 * curve used, grossed up by the fee.
 */
export function quoteBuy(
	curve: Curve,
	from: CurvePoint,
	quoteIn: number,
	seconds: number,
): BuyQuote {
	const rate = feeRateAt(seconds);
	const fee = quoteIn * rate;
	const net = quoteIn - fee;
	const room = TERMS.migrationQuote - from.quoteRaised;
	if (net < room) {
		return {
			paid: quoteIn,
			fee,
			base: pointAtQuote(curve, from.quoteRaised + net).baseSold -
				from.baseSold,
			completes: false,
		};
	}
	const paid = room / (1 - rate);
	return {
		paid,
		fee: paid - room,
		base: migrationPoint(curve).baseSold - from.baseSold,
		completes: true,
	};
}

export interface Graduation {
	/** Lamports taken as the migration fee. */
	migrationFee: number;
	poolQuote: number;
	poolBase: number;
	/** The pool's opening price, which is the curve's final price. */
	price: number;
	/** Base units the pool did not need, burned at graduation. */
	burned: number;
}

/** What graduation does with a completed launch (`docs/graduation.md`). */
export function graduate(curve: Curve): Graduation {
	const completed = migrationPoint(curve);
	const migrationFee = completed.quoteRaised * TERMS.migrationFeeRate;
	const poolQuote = completed.quoteRaised - migrationFee;
	const finalPrice = price(completed.sqrtPrice);
	const poolBase = poolQuote / finalPrice;
	return {
		migrationFee,
		poolQuote,
		poolBase,
		price: finalPrice,
		burned: TERMS.totalSupply - TERMS.creatorAllocation - completed.baseSold -
			poolBase,
	};
}

/** The pool's price after buyers take `bought` base units out of it. */
export function poolPriceAfter(graduation: Graduation, bought: number): number {
	const base = graduation.poolBase - bought;
	return (graduation.poolQuote * graduation.poolBase) / (base * base);
}

/** The plot: tokens sold (millions) across, SOL per million tokens up. */
export const PLOT = {
	width: 640,
	height: 400,
	left: 56,
	right: 16,
	top: 16,
	bottom: 44,
	tokens: [0, 900],
	price: [0, 0.7],
	tokenTicks: [200, 400, 600, 800],
	priceTicks: [0.1, 0.2, 0.3, 0.4, 0.5, 0.6],
} as const;

export function plotX(millionsSold: number): number {
	const [min, max] = PLOT.tokens;
	return PLOT.left +
		((millionsSold - min) / (max - min)) *
			(PLOT.width - PLOT.left - PLOT.right);
}

export function plotY(solPerMillion: number): number {
	const [min, max] = PLOT.price;
	return PLOT.top +
		(1 - (solPerMillion - min) / (max - min)) *
			(PLOT.height - PLOT.top - PLOT.bottom);
}

export function millionsAtPlotX(x: number): number {
	const [min, max] = PLOT.tokens;
	return min +
		((x - PLOT.left) / (PLOT.width - PLOT.left - PLOT.right)) * (max - min);
}

function plotPoint(point: CurvePoint): string {
	return `${plotX(tokens(point.baseSold) / 1e6).toFixed(2)},${
		plotY(solPerMillionTokens(price(point.sqrtPrice))).toFixed(2)
	}`;
}

/** SVG path of the curve from the start until `base` units are sold. */
export function curvePath(curve: Curve, base: number): string {
	const samples = 120;
	const points: string[] = [];
	for (let index = 0; index <= samples; index += 1) {
		points.push(plotPoint(pointAtBase(curve, (base * index) / samples)));
	}
	return `M${points.join("L")}`;
}

/** The shaded area under the curve up to `base` units sold: the SOL the curve holds. */
export function raisedAreaPath(curve: Curve, base: number): string {
	const baseline = plotY(PLOT.price[0]).toFixed(2);
	const end = plotX(tokens(base) / 1e6).toFixed(2);
	return `${curvePath(curve, base)}L${end},${baseline}L${
		plotX(0).toFixed(2)
	},${baseline}Z`;
}

/** SVG path of the graduated pool's price as buyers take tokens out of it, while it stays on the plot. */
export function poolPath(curve: Curve): string {
	const graduation = graduate(curve);
	const start = tokens(migrationPoint(curve).baseSold) / 1e6;
	const points: string[] = [];
	for (
		let bought = 0;
		bought < graduation.poolBase;
		bought += graduation.poolBase / 200
	) {
		const solPerMillion = solPerMillionTokens(
			poolPriceAfter(graduation, bought),
		);
		const millions = start + tokens(bought) / 1e6;
		if (solPerMillion > PLOT.price[1] || millions > PLOT.tokens[1]) {
			break;
		}
		points.push(
			`${plotX(millions).toFixed(2)},${plotY(solPerMillion).toFixed(2)}`,
		);
	}
	return `M${points.join("L")}`;
}

const formatters = new Map<number, Intl.NumberFormat>();

export function formatNumber(value: number, fractionDigits: number): string {
	let formatter = formatters.get(fractionDigits);
	if (formatter === undefined) {
		formatter = new Intl.NumberFormat("en-US", {
			maximumFractionDigits: fractionDigits,
			minimumFractionDigits: 0,
		});
		formatters.set(fractionDigits, formatter);
	}
	return formatter.format(value);
}

export function formatMillions(baseUnits: number): string {
	return `${formatNumber(tokens(baseUnits) / 1e6, 1)}M`;
}

/** The words the readout shows for a point on the curve and a 1 SOL buy from it. */
export interface LaunchSummary {
	raised: string;
	sold: string;
	price: string;
	buy: string;
}

export const PREVIEW_BUY = LAMPORTS_PER_SOL;

export function describeLaunch(
	curve: Curve,
	point: CurvePoint,
	seconds: number,
): LaunchSummary {
	const completed = point.quoteRaised >= TERMS.migrationQuote;
	const buy = quoteBuy(curve, point, PREVIEW_BUY, seconds);
	return {
		raised: `${formatNumber(sol(point.quoteRaised), 1)} of ${
			formatNumber(sol(TERMS.migrationQuote), 0)
		} SOL`,
		sold: `${formatMillions(point.baseSold)} tokens`,
		price: `${
			formatNumber(solPerMillionTokens(price(point.sqrtPrice)), 3)
		} SOL per 1M`,
		buy: completed
			? "Trading has stopped"
			: `${formatMillions(buy.base)} tokens after a ${
				formatNumber(feeRateAt(seconds) * 100, 1)
			}% fee`,
	};
}
