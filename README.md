# gemini
Google Gemini API proxy

### Deploy

- `Cloudflare Worker`

```shell
# Clone
git clone https://github.com/gemini.git && cd gemini

# Deploy to cloudflare worker
cd worker
npm install wrangler --save-dev
npx wrangler publish
```
