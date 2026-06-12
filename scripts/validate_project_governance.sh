#!/bin/sh
set -u

errors=0
warnings=0
temp_files=""

cleanup() {
  for temp_file in $temp_files; do
    [ -n "$temp_file" ] && [ -f "$temp_file" ] && rm -f "$temp_file"
  done
}

trap cleanup EXIT HUP INT TERM

make_temp() {
  temp_file="$(mktemp "${TMPDIR:-/tmp}/agent-governance-validator.XXXXXX")" || exit 2
  temp_files="${temp_files} ${temp_file}"
  printf '%s\n' "$temp_file"
}

warn() {
  printf 'WARNING: %s\n' "$1"
  warnings=$((warnings + 1))
}

error() {
  printf 'ERROR: %s\n' "$1"
  errors=$((errors + 1))
}

manifest_value() {
  awk -v key="$1" '
    $0 ~ "^[[:space:]]*" key ":[[:space:]]*" {
      sub("^[[:space:]]*" key ":[[:space:]]*", "")
      sub(/[[:space:]]*#.*/, "")
      gsub(/^["'\''"]|["'\''"]$/, "")
      print
      exit
    }
  ' "$manifest"
}

section_value() {
  awk -v section="$1" -v key="$2" '
    /^[A-Za-z0-9_]+:[[:space:]]*($|#)/ {
      current=$1
      sub(":", "", current)
      next
    }
    current == section && $0 ~ "^[[:space:]]{2}" key ":[[:space:]]*" {
      sub("^[[:space:]]*" key ":[[:space:]]*", "")
      sub(/[[:space:]]*#.*/, "")
      gsub(/^["'\''"]|["'\''"]$/, "")
      print
      exit
    }
  ' "$manifest"
}

section_pairs() {
  awk -v section="$1" '
    /^[A-Za-z0-9_]+:[[:space:]]*($|#)/ {
      current=$1
      sub(":", "", current)
      next
    }
    current == section && /^[[:space:]]{2}[A-Za-z0-9_]+:[[:space:]]*[^#[:space:]]/ {
      line=$0
      sub(/^[[:space:]]*/, "", line)
      sub(/[[:space:]]*#.*/, "", line)
      split(line, parts, ":")
      key=parts[1]
      sub("^[^:]+:[[:space:]]*", "", line)
      gsub(/^["'\''"]|["'\''"]$/, "", line)
      print key "\t" line
    }
  ' "$manifest"
}

capability_value() {
  section_value "capabilities" "$1"
}

contains_text() {
  [ -f "$1" ] && grep -Fq "$2" "$1"
}

claims_completion() {
  grep -Eiq '^[[:space:]*_>-]*status\b.*\b(complete|completed|done|shipped|delivered)\b|\b(COMPLETE|COMPLETED|DONE|SHIPPED|DELIVERED)\b' "$1"
}

shows_evidence() {
  grep -Eiq '```|\b(test|tests|tested|testing|cargo|npm|pnpm|yarn|pytest|gradle|mvn|make|go test|passed|passing|verified|verify|evidence|exit[[:space:]]*0|coverage|benchmark|smoke)\b' "$1"
}

check_capability_file() {
  capability="$1"
  shift
  [ "$(capability_value "$capability")" = "conformant" ] || return 0
  for relative_path in "$@"; do
    if [ ! -e "$root/$relative_path" ]; then
      error "$capability is conformant but required file is missing: $relative_path"
    fi
    if [ "$capability" != "task_router" ] && [ "$capability" != "evolution_feedback" ] && [ -f "$root/AGENTS.md" ] && ! grep -Fq "$relative_path" "$root/AGENTS.md"; then
      error "conformant recurring workflow is not routed from AGENTS.md: $capability -> $relative_path"
    fi
  done
}

usage() {
  printf 'Usage: %s <project-root>\n' "$0" >&2
}

if [ "$#" -ne 1 ]; then
  usage
  exit 2
fi

root="$(cd "$1" 2>/dev/null && pwd)"
if [ -z "${root:-}" ]; then
  printf 'ERROR: project root does not exist: %s\n' "$1" >&2
  exit 2
fi

manifest="$root/.agent-governance/manifest.yaml"
if [ ! -f "$manifest" ]; then
  error "missing .agent-governance/manifest.yaml"
  printf 'Governance validation failed: %d error(s), %d warning(s).\n' "$errors" "$warnings"
  exit 1
fi

profile="$(manifest_value profile)"
status="$(manifest_value status)"

while IFS="$(printf '\t')" read -r name target; do
  [ -n "$name" ] || continue
  if [ ! -e "$root/$target" ]; then
    error "declared entrypoint does not exist: $name -> $target"
  fi
done <<EOF
$(section_pairs entrypoints)
EOF

if [ "$profile" = "product" ] || [ "$profile" = "high-risk" ]; then
  for capability in task_router evolution_feedback testing_policy git_workflow requirement_intake iteration_workflow change_control; do
    if [ "$(capability_value "$capability")" = "not_applicable" ]; then
      error "$profile profile cannot mark $capability as not_applicable"
    fi
  done
  if [ ! -f "$root/docs/README.md" ]; then
    if [ "$status" = "conformant" ]; then
      error "product governance is missing documentation map: docs/README.md"
    else
      warn "product governance is missing documentation map: docs/README.md"
    fi
  fi
  if [ ! -f "$root/docs/sop/DOC-CHECK.md" ]; then
    if [ "$status" = "conformant" ]; then
      error "multi-layer product governance is missing docs/sop/DOC-CHECK.md"
    else
      warn "multi-layer product governance is missing docs/sop/DOC-CHECK.md"
    fi
  fi
fi

iteration_records=""
if [ -d "$root/docs/iterations" ]; then
  iteration_records="$(find "$root/docs/iterations" -maxdepth 1 -type f -name '*.md' ! -name 'README.md' ! -name 'readme.md' -print)"
fi
if [ -n "$iteration_records" ] && [ "$(capability_value iteration_workflow)" = "not_applicable" ]; then
  error "iteration records exist but iteration_workflow is marked not_applicable"
fi

if [ "$status" = "degraded" ] || [ "$status" = "adopting" ]; then
  warn "manifest status is '$status': declared capabilities are not fully trustworthy yet; verify they reflect reality before relying on the governance state"
fi

if [ -n "$iteration_records" ]; then
  while IFS= read -r record; do
    [ -n "$record" ] || continue
    if claims_completion "$record" && ! shows_evidence "$record"; then
      relative="${record#"$root/"}"
      warn "iteration claims completion but records no validation evidence (command, test, or recorded result): $relative"
    fi
  done <<EOF
$iteration_records
EOF
fi

board="$root/docs/BOARD.md"
if [ -f "$board" ]; then
  if ! grep -Eiq 'derived[[:space:]-]+operating[[:space:]-]+view' "$board"; then
    warn "docs/BOARD.md exists but is not explicitly marked as a derived operating view"
  fi
  if ! grep -Fq "Owner Doc" "$board"; then
    warn "docs/BOARD.md exists but does not include an Owner Doc column or equivalent label"
  fi
  if ! grep -Fq "Gate" "$board"; then
    warn "docs/BOARD.md exists but does not include a Gate column or equivalent label"
  fi
fi

check_capability_file task_router AGENTS.md
check_capability_file evolution_feedback EVOLUTION.md docs/sop/EVOLUTION-FEEDBACK.md
if [ "$(capability_value evolution_feedback)" = "conformant" ] && [ -f "$root/AGENTS.md" ] && ! grep -Fq "docs/sop/EVOLUTION-FEEDBACK.md" "$root/AGENTS.md"; then
  error "conformant evolution_feedback is not routed from AGENTS.md: docs/sop/EVOLUTION-FEEDBACK.md"
fi
check_capability_file testing_policy docs/sop/TESTING.md
check_capability_file git_workflow docs/sop/GIT-WORKFLOW.md
check_capability_file requirement_intake docs/sop/REQUIREMENT-INTAKE.md
check_capability_file iteration_workflow docs/sop/START-ITERATION.md docs/sop/ITERATION-WORKFLOW.md
check_capability_file change_control docs/sop/CHANGE-CONTROL.md
check_capability_file decision_records docs/decisions/README.md
check_capability_file release_workflow docs/sop/RELEASE.md

if [ -f "$root/AGENTS.md" ] && { [ "$profile" = "product" ] || [ "$profile" = "high-risk" ]; } && [ "$(capability_value task_router)" = "conformant" ]; then
  for section in "Hard Constraints" "Coding Behavior" "Git Rules" "Task Router" "Session End Checklist"; do
    if ! grep -Fq "$section" "$root/AGENTS.md"; then
      error "AGENTS.md is missing required section: $section"
    fi
  done
  if ! grep -Eq 'type\(scope\):[[:space:]]+description.*\[model:[[:space:]]*<model-name>\]' "$root/AGENTS.md"; then
    error "AGENTS.md Git Rules must include the Agent commit model tag format: type(scope): description (#story-id) [model:<model-name>]"
  fi
  if ! grep -Eq '\[model:[[:space:]]*<model-name>\].*(required|mandatory)|required.*\[model:[[:space:]]*<model-name>\]|mandatory.*\[model:[[:space:]]*<model-name>\]' "$root/AGENTS.md"; then
    error "AGENTS.md Git Rules must say the model tag is required for Agent-authored or Agent-assisted commits"
  fi
fi

markdown_files=""
for file in "$root/AGENTS.md" "$root/EVOLUTION.md" "$root/README.md"; do
  [ -f "$file" ] && markdown_files="${markdown_files}${file}
"
done
if [ -d "$root/docs" ]; then
  markdown_files="${markdown_files}$(find "$root/docs" -type f -name '*.md' -print)
"
fi

if [ -n "$markdown_files" ]; then
  while IFS= read -r source; do
    [ -n "$source" ] || continue
    link_file="$(make_temp)"
    grep -Eo '\[[^]]+\]\([^)#]+\.md(#[^)]+)?\)' "$source" | sed -E 's/^[^)]*\(([^)#]+\.md)(#[^)]+)?\)$/\1/' > "$link_file"
    while IFS= read -r link; do
      case "$link" in
        *://*|*'<'*) continue ;;
      esac
      source_dir="$(dirname "$source")"
      target="$(cd "$source_dir" 2>/dev/null && cd "$(dirname "$link")" 2>/dev/null && pwd)/$(basename "$link")"
      if [ ! -f "$target" ]; then
        error "broken Markdown link: ${source#"$root/"} -> $link"
      fi
    done < "$link_file"
  done <<EOF
$markdown_files
EOF
fi

active_files=""
[ -f "$root/AGENTS.md" ] && active_files="${active_files}${root}/AGENTS.md
"
for folder in "$root/docs/reference" "$root/docs/sop"; do
  if [ -d "$folder" ]; then
    active_files="${active_files}$(find "$folder" -type f -name '*.md' -print)
"
  fi
done

if [ -n "$active_files" ]; then
  while IFS= read -r source; do
    [ -n "$source" ] || continue
    ref_file="$(make_temp)"
    sed '/```/,/```/d' "$source" | grep -Eo '`src/[A-Za-z0-9_./@-]+\.[A-Za-z0-9]+`' | tr -d '`' | sort -u > "$ref_file"
    while IFS= read -r relative_path; do
      case "$relative_path" in
        *MyPage*|*Example*|*'<'*|*'>'*) continue ;;
      esac
      if [ ! -e "$root/$relative_path" ]; then
        error "missing explicit source path referenced by active governance: ${source#"$root/"} -> $relative_path"
      fi
    done < "$ref_file"
  done <<EOF
$active_files
EOF
fi

if [ "$errors" -gt 0 ]; then
  printf 'Governance validation failed: %d error(s), %d warning(s).\n' "$errors" "$warnings"
  exit 1
fi

printf 'Governance validation passed: %d warning(s).\n' "$warnings"
