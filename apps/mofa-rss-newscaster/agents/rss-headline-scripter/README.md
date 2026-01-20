# RSS Headline Scripter

Lightweight MoFA agent that converts RSS entries into short headline scripts.

- Input: `rss_feed` (JSON from rss-fetcher)
- Output: `news_scripts` (structured scripts for anchor-assigner)

This agent avoids external LLM calls to keep demo startup fast.
