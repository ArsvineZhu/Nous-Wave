# Full ChatGPT Data Export for Conversation Migration

This bundle cannot directly access the complete ChatGPT account conversation database or internal Project Memory store. For full conversation history, use ChatGPT's official data export.

Current official route (verified 2026-09-16):

1. Open ChatGPT **Settings**.
2. Open **Data controls**.
3. Choose **Export data** / **Export** and confirm.
4. When the export message arrives, download the ZIP while signed in to the same account. The download link is time-limited.
5. The export includes chat history and other relevant account data. For migration/reference in another personal ChatGPT account/workspace, extract `conversations.json` (or numbered conversation JSON files for large exports) and upload those files to a new chat/project as reference.

Important: uploading exported conversation JSON is not a full account merge. It does not recreate the old sidebar/chats and does not automatically transfer settings, Memory, GPTs, subscriptions, or workspace memberships.

Official references (titles only, URLs intentionally omitted from the migration bundle):
- OpenAI Help Center — “Exporting your ChatGPT history and data”
- OpenAI Help Center — “Transfer exported conversations between ChatGPT accounts”
