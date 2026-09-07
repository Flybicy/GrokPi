/**
 * pi-grok-skill-wiki — persistent skill-knowledge layer (WikiSkill-lite).
 *
 * Inspired by WikiSkill (Google Research, 2026): agent skills stay useful only
 * when the *insight* behind them survives across sessions. This extension keeps
 * the cheap 90% of that idea and drops the expensive evolutionary loop:
 *
 * - `patterns/<slug>.md`  wiki pages: reusable techniques, pitfalls, fixes.
 * - `index.md`            regenerated catalog of all pattern pages.
 * - `logs.md`             append-only evolution log (who recorded what, when).
 * - `inbox.md`            timestamped quick notes from `/skill-note`.
 *
 * The agent sees the wiki catalog in its system prompt (before_agent_start)
 * and records/reads knowledge through the skill_wiki_* tools.
 *
 * Storage root: $PI_GROK_SKILL_WIKI, else $GROK_HOME/skill-wiki,
 * else ~/.grok-pi/skill-wiki.
 */
import {
	existsSync,
	mkdirSync,
	readFileSync,
	writeFileSync,
	appendFileSync,
	readdirSync,
} from "node:fs";
import { homedir } from "node:os";
import { join } from "node:path";
import { Type } from "@sinclair/typebox";
import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";

const MAX_CATALOG_CHARS = 3000;

function wikiRoot(): string {
	const override = process.env.PI_GROK_SKILL_WIKI?.trim();
	if (override) return override;
	const grokHome = process.env.GROK_HOME?.trim();
	return join(grokHome || join(homedir(), ".grok-pi"), "skill-wiki");
}

function patternsDir(): string {
	return join(wikiRoot(), "patterns");
}

function ensureDirs(): void {
	mkdirSync(patternsDir(), { recursive: true });
}

function slugify(title: string): string {
	const slug = title
		.trim()
		.toLowerCase()
		.replace(/[^a-z0-9]+/g, "-")
		.replace(/^-+|-+$/g, "");
	return slug || "note";
}

function patternPagePath(slug: string): string {
	return join(patternsDir(), `${slug}.md`);
}

function logLine(text: string): void {
	appendFileSync(join(wikiRoot(), "logs.md"), `- ${new Date().toISOString()} ${text}\n`);
}

/** Rebuild index.md from current pattern pages. */
function rebuildIndex(): void {
	ensureDirs();
	const entries: string[] = [];
	for (const file of readdirSync(patternsDir())) {
		if (!file.endsWith(".md")) continue;
		const slug = file.slice(0, -3);
		let title = slug;
		let summary = "";
		try {
			const lines = readFileSync(patternPagePath(slug), "utf8").split(/\r?\n/);
			const heading = lines.find((line) => line.startsWith("# "));
			if (heading) title = heading.slice(2).trim();
			const body = lines
				.map((line) => line.trim())
				.filter((line) => line && !line.startsWith("#") && !line.startsWith("<!--"));
			summary = body.length > 0 ? body[0].slice(0, 120) : "";
		} catch {
			// skip unreadable page
		}
		entries.push(`- **${slug}** — ${title}${summary ? ` · ${summary}` : ""}`);
	}
	writeFileSync(
		join(wikiRoot(), "index.md"),
		`# Skill Wiki Catalog\n\n${entries.length === 0 ? "(empty — no recorded patterns yet)" : entries.sort().join("\n")}\n`,
		"utf8",
	);
}

function readCatalog(): string {
	try {
		return readFileSync(join(wikiRoot(), "index.md"), "utf8").trim();
	} catch {
		return "";
	}
}

function upsertPattern(slug: string, title: string, content: string): { created: boolean } {
	ensureDirs();
	const path = patternPagePath(slug);
	const created = !existsSync(path);
	const prev = created ? "" : readFileSync(path, "utf8");
	const merged = created
		? `# ${title}\n\n${content.trim()}\n`
		: `${prev.trimEnd()}\n\n---\n\n_Updated ${new Date().toISOString()}_\n\n${content.trim()}\n`;
	writeFileSync(path, merged, "utf8");
	rebuildIndex();
	logLine(`${created ? "recorded" : "updated"} pattern \`${slug}\``);
	return { created };
}

function searchPatterns(query: string): Array<{ slug: string; excerpt: string }> {
	ensureDirs();
	const needle = query.trim().toLowerCase();
	const hits: Array<{ slug: string; excerpt: string }> = [];
	if (!needle) return hits;
	for (const file of readdirSync(patternsDir())) {
		if (!file.endsWith(".md")) continue;
		const slug = file.slice(0, -3);
		let text = "";
		try {
			text = readFileSync(patternPagePath(slug), "utf8");
		} catch {
			continue;
		}
		const lower = text.toLowerCase();
		const at = lower.indexOf(needle);
		if (at < 0) continue;
		const start = Math.max(0, at - 80);
		const excerpt = text.slice(start, at + needle.length + 160).replace(/\s+/g, " ").trim();
		hits.push({ slug, excerpt });
	}
	return hits;
}

export default function (pi: ExtensionAPI) {
	// Inject the wiki catalog so the model knows prior lessons exist, and knows
	// it should persist new ones. Kept small; full pages load on demand.
	pi.on("before_agent_start", (event) => {
		let catalog = readCatalog();
		if (catalog.length > MAX_CATALOG_CHARS) {
			catalog = `${catalog.slice(0, MAX_CATALOG_CHARS)}\n...(truncated; use skill_wiki_search)`;
		}
		const wikiBlock = catalog
			? `<persistent-skill-wiki>\n${catalog}\n</persistent-skill-wiki>`
			: "<persistent-skill-wiki>(no recorded patterns yet)</persistent-skill-wiki>";
		const directive = [
			wikiBlock,
			"You have a persistent skill wiki that survives across sessions.",
			"When you run into a recurring pitfall or discover a reusable technique, record it with the skill_wiki_record tool.",
			"Consult the patterns above (skill_wiki_read / skill_wiki_search) before repeating past mistakes.",
		].join("\n");
		const base = event.systemPrompt ?? "";
		return { systemPrompt: `${base}\n\n${directive}` };
	});

	pi.registerTool({
		name: "skill_wiki_record",
		label: "Skill Wiki: record pattern",
		description:
			"Persist a reusable technique, pitfall, or fix into the cross-session skill wiki. Creates the pattern page or appends an update to it.",
		parameters: Type.Object({
			title: Type.String({ description: "Short pattern title, e.g. 'cargo-gnu-dlltool-broken'." }),
			content: Type.String({
				description: "What to remember: symptom, root cause, working fix or procedure.",
			}),
		}),
		async execute(_toolCallId, params: { title: string; content: string }) {
			const slug = slugify(params.title);
			const { created } = upsertPattern(slug, params.title, params.content);
			return {
				content: [
					{
						type: "text",
						text: `${created ? "Recorded" : "Updated"} pattern '${slug}' in the skill wiki. It will be visible in future sessions.`,
					},
				],
				details: { ok: true, slug, created },
			};
		},
	});

	pi.registerTool({
		name: "skill_wiki_read",
		label: "Skill Wiki: read pattern",
		description: "Read one skill-wiki pattern page in full by its slug.",
		parameters: Type.Object({
			slug: Type.String({ description: "Pattern slug from the wiki catalog." }),
		}),
		async execute(_toolCallId, params: { slug: string }) {
			const path = patternPagePath(params.slug);
			if (!existsSync(path)) {
				return {
					content: [{ type: "text", text: `No pattern named '${params.slug}'.` }],
					details: { ok: false, slug: params.slug },
				};
			}
			return {
				content: [{ type: "text", text: readFileSync(path, "utf8") }],
				details: { ok: true, slug: params.slug },
			};
		},
	});

	pi.registerTool({
		name: "skill_wiki_search",
		label: "Skill Wiki: search",
		description: "Full-text search over all skill-wiki pattern pages; returns matching slugs with excerpts.",
		parameters: Type.Object({
			query: Type.String({ description: "Keywords to search for." }),
		}),
		async execute(_toolCallId, params: { query: string }) {
			const hits = searchPatterns(params.query);
			if (hits.length === 0) {
				return {
					content: [{ type: "text", text: `No patterns match '${params.query}'.` }],
					details: { ok: true, hits: 0 },
				};
			}
			const text = hits.map((h) => `## ${h.slug}\n…${h.excerpt}…`).join("\n\n");
			return {
				content: [{ type: "text", text }],
				details: { ok: true, hits: hits.length },
			};
		},
	});

	// Quick human note: `/skill-note <text>` appends to inbox.md —
	// the agent folds inbox items into proper patterns with skill_wiki_record.
	pi.registerCommand("skill-note", {
		description: "Append a quick note to the skill-wiki inbox",
		handler: async (args, ctx) => {
			const text = String(args ?? "").trim();
			if (!text) {
				ctx.ui.notify("Usage: /skill-note <text>", "warning");
				return;
			}
			ensureDirs();
			appendFileSync(join(wikiRoot(), "inbox.md"), `- ${new Date().toISOString()} ${text}\n`);
			logLine("quick note added to inbox.md");
			ctx.ui.notify("Saved to skill-wiki inbox.", "info");
		},
	});

	pi.registerCommand("skill-wiki", {
		description: "Show skill-wiki status (pattern count, storage path)",
		handler: async (_args, ctx) => {
			ensureDirs();
			const count = readdirSync(patternsDir()).filter((f) => f.endsWith(".md")).length;
			ctx.ui.notify(
				`Skill wiki: ${count} pattern(s) at ${wikiRoot()}\nQuick note: /skill-note <text>`,
				"info",
			);
		},
	});
}
