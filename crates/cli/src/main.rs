use std::collections::BTreeMap;
use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use cottonmd_graph::VaultGraph;
use cottonmd_markdown::{parse_note, ParsedNote};
use cottonmd_search::{IndexedDocument, SearchIndex};
use cottonmd_vault::{EntryKind, Vault, VaultFile};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() || args[0] == "--help" || args[0] == "-h" {
        print_usage();
        return Ok(());
    }

    let mut vault_path = env::current_dir().map_err(|e| e.to_string())?;
    let mut pos = 0;
    if args.first().map(String::as_str) == Some("--vault") {
        let dir = args
            .get(1)
            .ok_or_else(|| "missing directory after --vault".to_string())?;
        vault_path = PathBuf::from(dir);
        pos = 2;
    }

    let cmd = args
        .get(pos)
        .map(String::as_str)
        .ok_or_else(|| "missing subcommand (run with --help)".to_string())?;
    let rest = &args[pos + 1..];

    let vault = Vault::open(&vault_path)
        .map_err(|e| format!("failed to open vault {}: {e}", vault_path.display()))?;
    let files = vault.list_files().map_err(|e| e.to_string())?;

    match cmd {
        "files" => {
            for f in &files {
                println!("{}\t({:?}, {} B)", f.relative_path, f.kind, f.size_bytes);
            }
        }
        "read" => {
            let target = rest
                .first()
                .ok_or_else(|| "usage: read <relative-path>".to_string())?;
            let content = vault.read_to_string(target).map_err(|e| e.to_string())?;
            print!("{content}");
        }
        "stats" => {
            let (parsed, graph) = index_vault(&vault, &files)?;
            let total_links: usize = parsed.values().map(|p| p.links.len()).sum();
            let total_tasks: usize = parsed.values().map(|p| p.tasks.len()).sum();
            println!("Vault:       {}", vault.root().display());
            println!("Files:       {}", files.len());
            println!("Markdown:    {}", graph.note_count());
            println!("Links:       {total_links}");
            println!("Tags:        {}", graph.tags().len());
            println!("Tasks:       {total_tasks}");
            println!("Orphans:     {}", graph.orphans().len());
            println!("Deadends:    {}", graph.deadends().len());
            println!("Unresolved:  {}", graph.unresolved_links().len());
        }
        "search" => {
            let query = rest.join(" ");
            if query.trim().is_empty() {
                return Err("usage: search <query>".into());
            }
            let mut index = SearchIndex::default();
            for f in files.iter().filter(|f| f.kind == EntryKind::Markdown) {
                if let Ok(body) = vault.read_to_string(&f.relative_path) {
                    index.insert(IndexedDocument {
                        path: f.relative_path.clone(),
                        title: f.stem.clone(),
                        body,
                    });
                }
            }
            for hit in index.search(&query, 50) {
                println!("{}:{}  {}", hit.path, hit.line, hit.snippet);
            }
        }
        "links" => {
            let target = rest
                .first()
                .ok_or_else(|| "usage: links <relative-path>".to_string())?;
            let (_, graph) = index_vault(&vault, &files)?;
            for link in graph.outgoing_links(target) {
                println!("{link}");
            }
        }
        "backlinks" => {
            let target = rest
                .first()
                .ok_or_else(|| "usage: backlinks <relative-path>".to_string())?;
            let (_, graph) = index_vault(&vault, &files)?;
            for link in graph.backlinks(target) {
                println!("{link}");
            }
        }
        "orphans" => {
            let (_, graph) = index_vault(&vault, &files)?;
            for note in graph.orphans() {
                println!("{note}");
            }
        }
        "deadends" => {
            let (_, graph) = index_vault(&vault, &files)?;
            for note in graph.deadends() {
                println!("{note}");
            }
        }
        "unresolved" => {
            let (_, graph) = index_vault(&vault, &files)?;
            for (src, targets) in graph.unresolved_links() {
                for t in targets {
                    println!("{src} -> [[{t}]]");
                }
            }
        }
        "tags" => {
            let (_, graph) = index_vault(&vault, &files)?;
            for (tag, notes) in graph.tags() {
                println!("#{tag}\t({})", notes.len());
            }
        }
        "tasks" => {
            let (parsed, _) = index_vault(&vault, &files)?;
            for (path, note) in &parsed {
                for task in &note.tasks {
                    println!("{path}:{} [{}] {}", task.line, task.status, task.text);
                }
            }
        }
        other => return Err(format!("unknown subcommand: {other} (run with --help)")),
    }

    Ok(())
}

fn index_vault(
    vault: &Vault,
    files: &[VaultFile],
) -> Result<(BTreeMap<String, ParsedNote>, VaultGraph), String> {
    let mut parsed = BTreeMap::new();
    for file in files.iter().filter(|f| f.kind == EntryKind::Markdown) {
        let text = vault
            .read_to_string(&file.relative_path)
            .map_err(|e| format!("read {}: {e}", file.relative_path))?;
        parsed.insert(file.relative_path.clone(), parse_note(&text));
    }
    let graph = VaultGraph::build(
        files,
        parsed.iter().map(|(k, v)| (k.as_str(), v)),
    );
    Ok((parsed, graph))
}

fn print_usage() {
    println!(
        "\
cottonmd-cli — Fast local-first Markdown vault CLI

USAGE:
  cottonmd-cli [--vault <dir>] <command> [args]

COMMANDS:
  stats                 Show vault file, link, tag, and graph counts
  files                 List indexed files inside the vault
  read <path>           Print a vault file
  search <query>        Search note titles and bodies
  links <path>          List resolved outgoing wikilinks
  backlinks <path>      List incoming backlinks
  orphans               List notes with no incoming links
  deadends              List notes with no outgoing links
  unresolved            List broken/unresolved wikilinks
  tags                  List tags with note counts
  tasks                 List Markdown checklist items"
    );
}
