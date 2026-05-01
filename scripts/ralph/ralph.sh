#!/bin/bash
# Ralph Wiggum - Long-running AI agent loop
# Usage: ./ralph.sh [max_iterations] [cli_tool] [model] [share]
# cli_tool: amp (default), opencode, or pi
# model: opencode model ID, amp mode (smart/rush), or pi model pattern
# share: true/false (default: false) - share session for opencode/pi

set -e

# Show help
show_help() {
	cat << 'EOF'
Ralph Wiggum - Long-running AI agent loop for EchoNote

Usage:
  ./ralph.sh [max_iterations] [cli_tool] [model] [share]

Arguments:
  max_iterations    Number of iterations to run (default: 10)
  cli_tool         CLI tool to use: amp (default), opencode, or pi
  model            Model ID for opencode, mode for amp (smart/rush), or pi model pattern
  share            Share session: true/false (default: false) - only for opencode

Options:
  -h, --help       Show this help message and exit

Examples:
  # Run with defaults (amp, 10 iterations)
  ./ralph.sh

  # Run 5 iterations with opencode
  ./ralph.sh 5 opencode

  # Run with specific model
  ./ralph.sh 10 opencode opencode/big-pickle true

  # Run with pi (uses --model flag)
  ./ralph.sh 10 pi google/gemini-2.0-flash

  # Run pi with thinking level
  ./ralph.sh 10 pi claude-sonnet:high

Files:
  prompt-amp.md       - System prompt for amp CLI
  prompt-opencode.md  - System prompt for opencode CLI
  prompt-pi.md        - System prompt for pi CLI
  prd.json            - Product requirements in Ralph format
  progress.txt        - Progress log of completed stories

Completion Signal:
  Ralph stops when the agent outputs: <promise>COMPLETE</promise>
EOF
}

# Parse arguments for --help before positional args
for arg in "$@"; do
	if [ "$arg" = "--help" ] || [ "$arg" = "-h" ]; then
		show_help
		exit 0
	fi
done

MAX_ITERATIONS=${1:-10}
CLI_TOOL=${2:-amp}
MODEL=${3:-}
SHARE=${4:-false}
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROMPT_FILE="$SCRIPT_DIR/prompt-$CLI_TOOL.md"

# Set opencode permissions via environment variable (equivalent to --dangerously-allow-all)
if [ "$CLI_TOOL" = "opencode" ]; then
	export OPENCODE_PERMISSION='{"*": "allow"}'
	export OPENCODE_DISABLE_AUTOCOMPACT=true
fi

# Set pi permissions via environment variable (equivalent to --dangerously-allow-all)
if [ "$CLI_TOOL" = "pi" ]; then
	export PI_PERMISSION='{"*": "allow"}'
fi

PRD_FILE="$SCRIPT_DIR/prd.json"
PROGRESS_FILE="$SCRIPT_DIR/progress.txt"
ARCHIVE_DIR="$SCRIPT_DIR/archive"
LAST_BRANCH_FILE="$SCRIPT_DIR/.last-branch"

# Archive previous run if branch changed
if [ -f "$PRD_FILE" ] && [ -f "$LAST_BRANCH_FILE" ]; then
	CURRENT_BRANCH=$(jq -r '.branchName // empty' "$PRD_FILE" 2>/dev/null || echo "")
	LAST_BRANCH=$(cat "$LAST_BRANCH_FILE" 2>/dev/null || echo "")

	if [ -n "$CURRENT_BRANCH" ] && [ -n "$LAST_BRANCH" ] && [ "$CURRENT_BRANCH" != "$LAST_BRANCH" ]; then
		# Archive the previous run
		DATE=$(date +%Y-%m-%d)
		# Strip "ralph/" prefix from branch name for folder
		FOLDER_NAME=$(echo "$LAST_BRANCH" | sed 's|^ralph/||')
		ARCHIVE_FOLDER="$ARCHIVE_DIR/$DATE-$FOLDER_NAME"

		echo "Archiving previous run: $LAST_BRANCH"
		mkdir -p "$ARCHIVE_FOLDER"
		[ -f "$PRD_FILE" ] && cp "$PRD_FILE" "$ARCHIVE_FOLDER/"
		[ -f "$PROGRESS_FILE" ] && cp "$PROGRESS_FILE" "$ARCHIVE_FOLDER/"
		echo "   Archived to: $ARCHIVE_FOLDER"

		# Reset progress file for new run
		echo "# Ralph Progress Log" >"$PROGRESS_FILE"
		echo "Started: $(date)" >>"$PROGRESS_FILE"
		echo "---" >>"$PROGRESS_FILE"
	fi
fi

# Track current branch
if [ -f "$PRD_FILE" ]; then
	CURRENT_BRANCH=$(jq -r '.branchName // empty' "$PRD_FILE" 2>/dev/null || echo "")
	if [ -n "$CURRENT_BRANCH" ]; then
		echo "$CURRENT_BRANCH" >"$LAST_BRANCH_FILE"
	fi
fi

# Initialize progress file if it doesn't exist
if [ ! -f "$PROGRESS_FILE" ]; then
	echo "# Ralph Progress Log" >"$PROGRESS_FILE"
	echo "Started: $(date)" >>"$PROGRESS_FILE"
	echo "---" >>"$PROGRESS_FILE"
fi

echo "Starting Ralph - Max iterations: $MAX_ITERATIONS"
if [ -n "$MODEL" ]; then
	echo "Using CLI: $CLI_TOOL (model: $MODEL)"
else
	echo "Using CLI: $CLI_TOOL (default model)"
fi
if [ "$CLI_TOOL" = "opencode" ]; then
	echo "Share session: $SHARE"
fi

for i in $(seq 1 $MAX_ITERATIONS); do
	echo ""
	echo "═══════════════════════════════════════════════════════"
	echo "  Ralph Iteration $i of $MAX_ITERATIONS"
	echo "═══════════════════════════════════════════════════════"

	# Run amp, opencode, or pi with the ralph prompt
	if [ "$CLI_TOOL" = "opencode" ]; then
		OPENCODE_MODEL=${MODEL:-opencode/big-pickle}
		if [ "$SHARE" = "true" ]; then
			OUTPUT=$(cat "$PROMPT_FILE" | opencode run -m "$OPENCODE_MODEL" --agent build --share - 2>&1 | tee /dev/stderr) || true
		else
			OUTPUT=$(cat "$PROMPT_FILE" | opencode run -m "$OPENCODE_MODEL" --agent build - 2>&1 | tee /dev/stderr) || true
		fi
	elif [ "$CLI_TOOL" = "pi" ]; then
		# pi uses --model pattern and supports thinking levels via :suffix
		if [ -n "$MODEL" ]; then
			OUTPUT=$(cat "$PROMPT_FILE" | pi --model "$MODEL" -p 2>&1 | tee /dev/stderr) || true
		else
			OUTPUT=$(cat "$PROMPT_FILE" | pi -p 2>&1 | tee /dev/stderr) || true
		fi
	else
		if [ -n "$MODEL" ]; then
			OUTPUT=$(cat "$PROMPT_FILE" | amp --dangerously-allow-all --mode "$MODEL" 2>&1 | tee /dev/stderr) || true
		else
			OUTPUT=$(cat "$PROMPT_FILE" | amp --dangerously-allow-all 2>&1 | tee /dev/stderr) || true
		fi
	fi

	# Check for completion signal
	if echo "$OUTPUT" | grep -q "<promise>COMPLETE</promise>"; then
		echo ""
		echo "Ralph completed all tasks!"
		echo "Completed at iteration $i of $MAX_ITERATIONS"
		exit 0
	fi

	echo "Iteration $i complete. Continuing..."
	sleep 2
done

echo ""
echo "Ralph reached max iterations ($MAX_ITERATIONS) without completing all tasks."
echo "Check $PROGRESS_FILE for status."
exit 1
