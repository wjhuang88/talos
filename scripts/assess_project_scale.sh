#!/bin/sh
set -eu

root="${1:-.}"
root=$(cd "$root" 2>/dev/null && pwd) || {
  printf 'ERROR: project root does not exist: %s\n' "$1" >&2
  exit 2
}

count_files() {
  find "$root" \
    -path "$root/.git" -prune -o \
    -path "$root/.codex" -prune -o \
    -path "$root/.opencode" -prune -o \
    -path "$root/.sisyphus" -prune -o \
    -path "$root/node_modules" -prune -o \
    -path "$root/target" -prune -o \
    -path "$root/dist" -prune -o \
    -path "$root/build" -prune -o \
    -type f "$@" -print 2>/dev/null | wc -l | tr -d ' '
}

has_path() {
  [ -e "$root/$1" ]
}

has_any_path() {
  for path in "$@"; do
    has_path "$path" && return 0
  done
  return 1
}

has_text() {
  pattern="$1"
  find "$root" \
    -path "$root/.git" -prune -o \
    -path "$root/.codex" -prune -o \
    -path "$root/.opencode" -prune -o \
    -path "$root/.sisyphus" -prune -o \
    -path "$root/node_modules" -prune -o \
    -path "$root/target" -prune -o \
    -type f \( -name '*.js' -o -name '*.ts' -o -name '*.tsx' -o -name '*.py' -o -name '*.go' -o -name '*.rs' -o -name '*.java' -o -name '*.rb' -o -name '*.php' -o -name '*.cs' -o -name '*.json' -o -name '*.toml' \) -print 2>/dev/null |
    xargs grep -Eiq "$pattern" 2>/dev/null
}

source_files=$(count_files \( -name '*.js' -o -name '*.ts' -o -name '*.tsx' -o -name '*.py' -o -name '*.go' -o -name '*.rs' -o -name '*.java' -o -name '*.rb' -o -name '*.php' -o -name '*.cs' \))
package_files=$(count_files \( -name 'package.json' -o -name 'Cargo.toml' -o -name 'go.mod' -o -name 'pyproject.toml' -o -name 'pom.xml' -o -name '*.csproj' \))
ci_workflows=$(count_files -path "$root/.github/workflows/*")

release_signal=false
has_any_path "releases" ".github/workflows/release.yml" "CHANGELOG.md" && release_signal=true
if git -C "$root" tag --list 'v[0-9]*' >/tmp/agent-scale-tags.$$ 2>/dev/null; then
  [ -s /tmp/agent-scale-tags.$$ ] && release_signal=true
fi
rm -f /tmp/agent-scale-tags.$$

migration_signal=false
has_any_path "db/migrations" "migrations" "prisma/migrations" "database/migrations" && migration_signal=true

auth_signal=false
has_text '(^|[^A-Za-z])(auth|authentication|authorization|permission|permissions|rbac|oauth|jwt|session|token)([^A-Za-z]|$)|auth[-_]' && auth_signal=true

external_signal=false
has_text 'webhook|payment|stripe|email|smtp|queue|kafka|sqs|llm|tool executor|third[-_ ]party|api client' && external_signal=true

planning_docs=0
for path in "docs/backlog" "docs/iterations" "docs/roadmap" "docs/decisions" "docs/proposals"; do
  has_path "$path" && planning_docs=$((planning_docs + 1))
done

worktree_count=0
if git -C "$root" rev-parse --git-dir >/dev/null 2>&1; then
  worktree_count=$(git -C "$root" worktree list 2>/dev/null | wc -l | tr -d ' ')
fi

product_signals=0
[ "$source_files" -gt 80 ] && product_signals=$((product_signals + 1))
[ "$package_files" -gt 2 ] && product_signals=$((product_signals + 1))
[ "$ci_workflows" -gt 0 ] && product_signals=$((product_signals + 1))
[ "$planning_docs" -ge 2 ] && product_signals=$((product_signals + 1))
[ "$release_signal" = true ] && product_signals=$((product_signals + 1))

high_risk_signals=0
[ "$migration_signal" = true ] && [ "$release_signal" = true ] && high_risk_signals=$((high_risk_signals + 1))
[ "$auth_signal" = true ] && high_risk_signals=$((high_risk_signals + 1))
[ "$external_signal" = true ] && high_risk_signals=$((high_risk_signals + 1))

profile=minimal
[ "$product_signals" -ge 2 ] && profile=product
[ "$high_risk_signals" -gt 0 ] && profile=high-risk

branch_mode=simple
[ "$release_signal" = true ] && branch_mode=release-managed

worktree_mode=none
if [ "$branch_mode" = release-managed ] || [ "$profile" = high-risk ]; then
  worktree_mode=on-demand
fi
[ "$worktree_count" -gt 1 ] && worktree_mode=required

cat <<EOF
recommended_profile: $profile
recommended_branch_mode: $branch_mode
recommended_worktree_mode: $worktree_mode

signals:
  source_files: $source_files
  package_files: $package_files
  ci_workflows: $ci_workflows
  release_signal: $release_signal
  migration_signal: $migration_signal
  auth_signal: $auth_signal
  external_signal: $external_signal
  planning_doc_groups: $planning_docs
  git_worktrees: $worktree_count

scores:
  product_signals: $product_signals
  high_risk_signals: $high_risk_signals
EOF
