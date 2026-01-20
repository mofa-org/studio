#!/usr/bin/env python3
"""
RSS Headline Scripter Agent - Converts RSS entries into headline scripts.

This agent avoids external LLM calls to keep demo startup fast.
"""
import json
import os
import re
from datetime import datetime
from typing import Any, Dict

from mofa.agent_build.base.base_agent import MofaAgent, run_agent

TAG_RE = re.compile(r"<[^>]+>")
WHITESPACE_RE = re.compile(r"\s+")


def create_error_response(error_type: str, message: str, details: dict = None) -> dict:
    """Create standardized error response."""
    return {
        "error": True,
        "error_type": error_type,
        "message": message,
        "details": details or {},
    }


def clean_text(value: str) -> str:
    """Strip HTML tags and normalize whitespace."""
    if not value:
        return ""
    text = TAG_RE.sub(" ", value)
    text = WHITESPACE_RE.sub(" ", text)
    return text.strip()


def build_script(entry: Dict[str, Any], index: int) -> dict:
    """Create a short headline script from an RSS entry."""
    title = (entry.get("title") or "").strip()
    description = entry.get("description") or entry.get("summary") or ""
    summary = clean_text(description)
    if len(summary) > 240:
        summary = summary[:240].rstrip() + "..."

    parts = []
    if title:
        parts.append(f"Headline: {title}.")
    if summary:
        parts.append(f"Summary: {summary}")

    script_text = " ".join(parts) if parts else "Headline: (no title)."

    return {
        "id": entry.get("id") or entry.get("link") or f"entry_{index}",
        "title": title,
        "script": script_text,
        "link": entry.get("link", ""),
        "published": entry.get("published", ""),
        "content_fetched": False,
        "content_length": 0,
        "generated_at": datetime.utcnow().isoformat() + "Z",
    }


def process_feed_to_scripts(feed_data: dict) -> dict:
    """Convert feed data into headline scripts."""
    if feed_data.get("error"):
        return feed_data

    entries = feed_data.get("entries", [])
    if not entries:
        return create_error_response("empty_feed", "No entries found in feed data", {})

    feed_info = feed_data.get("feed", {})
    feed_title = feed_info.get("title", "Unknown Feed")

    max_entries = int(os.getenv("MAX_ENTRIES", "10"))
    entries_to_process = entries[:max_entries]

    scripts = [build_script(entry, idx) for idx, entry in enumerate(entries_to_process, start=1)]

    return {
        "error": False,
        "feed_title": feed_title,
        "feed_url": feed_data.get("url", ""),
        "scripts": scripts,
        "script_count": len(scripts),
        "generated_at": datetime.utcnow().isoformat() + "Z",
    }


def parse_input(input_data: Any) -> dict:
    """Parse input data to extract feed dict."""
    if isinstance(input_data, str):
        try:
            parsed = json.loads(input_data)
            if isinstance(parsed, dict):
                return parsed
            return create_error_response("parse_error", "Invalid feed data", {})
        except json.JSONDecodeError:
            return create_error_response("parse_error", "Invalid JSON input", {})
    if isinstance(input_data, dict):
        return input_data
    return create_error_response("parse_error", "Unsupported input type", {})


def process_request(input_data: Any) -> str:
    """Process RSS feed into headline scripts and return JSON string."""
    feed_data = parse_input(input_data)
    result = process_feed_to_scripts(feed_data)
    return json.dumps(result, ensure_ascii=False)


@run_agent
def run(agent: MofaAgent):
    """Main agent run loop."""
    agent.write_log("Headline scripter agent started")

    input_data = agent.receive_parameter("rss_feed")
    result_json = process_request(input_data)

    agent.send_output(agent_output_name="news_scripts", agent_result=result_json)

    agent.write_log("Headline scripter agent completed")


def main():
    """Main entry point."""
    agent = MofaAgent(agent_name="rss-headline-scripter", is_write_log=True)
    run(agent=agent)


if __name__ == "__main__":
    main()
