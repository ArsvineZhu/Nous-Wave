# Nous Wave Workspace Migration Bundle

Created: 2026-09-16
Purpose: migrate the currently recoverable Nous Wave project workspace/context into another ChatGPT workspace or project.

## What is included

- `data_sources/` — the four distinct project data-source conversation objects, disambiguated by creation timestamp and original file ID so same-named sources cannot overwrite each other.
- `raw_workspace/` — files that were physically present in the active runtime and could therefore be copied byte-for-byte.
- `recovered_context/` — a self-contained recovery checkpoint of the current Nous Wave architectural state and recent NousQL decisions reconstructed from accessible project context and files.
- `indexes/` — inventory of additional Nous Wave files found in ChatGPT File Library, including creation metadata and migration status.
- `instructions/` — instructions for obtaining a full ChatGPT account data export when complete conversation history is required.
- `MANIFEST.json` and `SHA256SUMS.txt` — bundle inventory and hashes.

## Important limitation

The ChatGPT File Library interface available to this session permits search/read access but does not expose the original file bytes for arbitrary File Library objects. The four project data-source conversation objects are nevertheless included individually: two as byte-exact runtime copies and two reconstructed from the complete readable File Library content, with recovery status recorded in `data_sources/SOURCE_CONVERSATIONS_INDEX.*`. Therefore, only files physically mounted in the active runtime are included as exact originals. Additional discovered files are indexed and their high-value project state is preserved in `recovered_context/PROJECT_RECOVERY_CHECKPOINT.md`, but they are not falsely represented as byte-identical exports.

Likewise, this session cannot directly dump the complete internal ChatGPT Project Memory store or all historical chats. The official ChatGPT Data Export is the authoritative route for a full account-level conversation export.

## Suggested migration order

1. Upload this ZIP (or its extracted contents) to the new Nous Wave workspace.
2. Read `recovered_context/PROJECT_RECOVERY_CHECKPOINT.md` first.
3. Preserve the `raw_workspace/` files as historical evidence rather than current architecture authority.
4. If full prior chat history is needed, follow `instructions/FULL_CHATGPT_DATA_EXPORT.md` and upload the exported conversation JSON files to the new workspace as reference.
