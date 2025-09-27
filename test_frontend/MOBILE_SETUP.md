# 📱 Mobile Testing Setup

The audio handshake app requires **HTTPS** for microphone access on mobile devices. Here are quick ways to test on your phone:

## Option 1: Using ngrok (Recommended)

1. Install ngrok: `npm install -g ngrok` or download from [ngrok.com](https://ngrok.com)
2. Start your dev server: `npm run dev`
3. In a new terminal: `ngrok http 3000`
4. Use the HTTPS URL provided (e.g., `https://abc123.ngrok.io`)

## Option 2: Using local-ssl-proxy

```bash
npm install -g local-ssl-proxy
npm run dev  # Start Next.js on port 3000
local-ssl-proxy --source 3001 --target 3000
```

Then visit `https://localhost:3001` on your phone (you'll need to accept the self-signed certificate).

## Option 3: Deploy to Vercel/Netlify

Quick deployment for testing:

```bash
# Vercel
npm install -g vercel
vercel

# Netlify
npm install -g netlify-cli
netlify deploy
```

## Testing Tips

1. **Chrome Mobile**: Works best, supports all Web Audio features
2. **Safari Mobile**: Good compatibility, may need user gesture
3. **Firefox Mobile**: Basic support, some limitations
4. **Permissions**: Always allow microphone access when prompted
5. **Distance**: Keep devices within 2-3 feet for best results
6. **Volume**: Ensure device volume is up (ultrasonic frequencies need adequate volume)

## Common Issues

- **"navigator.mediaDevices is undefined"**: You need HTTPS
- **Permission denied**: Check browser microphone settings
- **No audio detected**: Try refreshing or increasing volume
- **iOS Safari**: May require user interaction before audio works

## Local Network Testing

If using your local IP (like `192.168.1.100:3000`), it won't work on mobile due to HTTPS requirement. Use one of the HTTPS options above instead.
