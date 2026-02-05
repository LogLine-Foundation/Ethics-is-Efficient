# Deploying logline.foundation to Cloudflare Pages

This guide walks you through deploying the logline.foundation website to Cloudflare Pages.

## Prerequisites

- A Cloudflare account (free tier works)
- Access to the GitHub repository
- Domain ownership for logline.foundation (if using custom domain)

## Step 1: Create a Cloudflare Pages Project

1. Log in to your Cloudflare account at https://dash.cloudflare.com
2. Navigate to **Pages** in the left sidebar
3. Click **Create a project**
4. Choose **Connect to Git**

## Step 2: Connect GitHub Repository

1. Select **GitHub** as your Git provider
2. Authorize Cloudflare to access your GitHub account (if not already done)
3. Select the repository: `LogLine-Foundation/Ethics-is-Efficient`
4. Click **Begin setup**

## Step 3: Configure Build Settings

Configure the following settings for your project:

### Project Name
```
logline-foundation
```

### Production Branch
```
main
```

### Build Settings

- **Framework preset:** None
- **Build command:** (leave empty)
- **Build output directory:** `/website`

### Environment Variables
No environment variables are required.

## Step 4: Deploy

1. Review your settings
2. Click **Save and Deploy**
3. Cloudflare will begin the deployment process
4. Your site will be available at `https://logline-foundation.pages.dev`

Initial deployment typically takes 1-2 minutes.

## Step 5: Configure Custom Domain (Optional)

To use the custom domain `logline.foundation`:

### If Domain is Already on Cloudflare

1. In your Pages project, go to **Custom domains**
2. Click **Set up a custom domain**
3. Enter: `logline.foundation`
4. Click **Continue**
5. Cloudflare will automatically configure DNS records
6. Click **Activate domain**

The domain should be active within a few minutes.

### If Domain is NOT on Cloudflare

1. In your Pages project, go to **Custom domains**
2. Click **Set up a custom domain**
3. Enter: `logline.foundation`
4. Cloudflare will provide DNS records (CNAME or A records)
5. Add these records to your domain registrar's DNS settings
6. Wait for DNS propagation (can take up to 48 hours)

### Add www Subdomain (Recommended)

1. In **Custom domains**, click **Set up a custom domain** again
2. Enter: `www.logline.foundation`
3. Follow the same steps as above

## Step 6: Verify Deployment

After deployment completes:

1. Visit your site at `https://logline-foundation.pages.dev`
2. If using custom domain, also check `https://logline.foundation`
3. Verify all sections load correctly:
   - Header and navigation
   - Papers section with all 9 papers
   - Install section with code blocks
   - Verify section
   - About section
4. Test redirects:
   - `/papers` should redirect to GitHub papers directory
   - `/github` should redirect to main repository
   - `/repo` should redirect to main repository
5. Check mobile responsiveness by resizing your browser

## Automatic Deployments

Cloudflare Pages is now configured to automatically deploy:

- **Production:** Every push to the `main` branch
- **Preview:** Every pull request creates a preview deployment

You'll see deployment status in:
- Cloudflare Pages dashboard
- GitHub commit status checks
- GitHub pull request comments

## Deployment Settings

The site uses these configuration files:

- **`_headers`**: Security headers (CSP, X-Frame-Options, etc.)
- **`_redirects`**: URL redirects for `/papers`, `/github`, `/repo`
- **`index.html`**: Single-file website (no build step required)

## Security Headers

The following security headers are automatically applied via `_headers`:

- **X-Frame-Options:** DENY (prevents clickjacking)
- **X-Content-Type-Options:** nosniff (prevents MIME sniffing)
- **Referrer-Policy:** strict-origin-when-cross-origin
- **Permissions-Policy:** Restricts geolocation, microphone, camera
- **X-XSS-Protection:** Enabled with blocking mode
- **Content-Security-Policy:** Restricts resource loading

## Troubleshooting

### Build Fails

If the build fails:
1. Verify the build output directory is set to `/website`
2. Ensure the build command is empty
3. Check that all files exist in the `website/` directory

### Custom Domain Not Working

If custom domain doesn't work:
1. Verify DNS records are correct in Cloudflare
2. Check DNS propagation: https://dnschecker.org/
3. Ensure SSL/TLS encryption mode is set to "Full" or "Full (strict)"
4. Wait up to 48 hours for full DNS propagation

### Redirects Not Working

If redirects don't work:
1. Verify `_redirects` file exists in `/website` directory
2. Check the file format (space-separated, no tabs)
3. Ensure status codes are correct (302 for temporary)

### Security Headers Not Applied

If security headers aren't working:
1. Verify `_headers` file exists in `/website` directory
2. Check file format (no extra whitespace)
3. Test headers using browser dev tools or: https://securityheaders.com/

## Performance

The website is optimized for performance:

- Single HTML file (no external dependencies)
- Inline CSS (no separate stylesheet requests)
- No JavaScript (instant page loads)
- Minimal asset size (~18KB total)
- Cloudflare's global CDN for fast delivery

Expected metrics:
- **First Contentful Paint:** < 0.5s
- **Time to Interactive:** < 0.5s
- **Lighthouse Performance Score:** 95-100

## Monitoring

Monitor your deployment:

1. **Cloudflare Analytics:** Available in Pages project dashboard
2. **Deployment History:** View all deployments and their status
3. **Build Logs:** Check logs if deployments fail

## Updating the Site

To update the website:

1. Make changes to files in the `website/` directory
2. Commit and push to the `main` branch
3. Cloudflare automatically deploys within 1-2 minutes
4. Check the deployment status in Cloudflare Pages dashboard

## Support

For issues or questions:

- **Cloudflare Pages Documentation:** https://developers.cloudflare.com/pages/
- **GitHub Issues:** https://github.com/LogLine-Foundation/Ethics-is-Efficient/issues
- **Cloudflare Community:** https://community.cloudflare.com/

## Summary Checklist

- [ ] Cloudflare Pages project created
- [ ] GitHub repository connected
- [ ] Build settings configured (no build command, output: `/website`)
- [ ] Initial deployment successful
- [ ] Custom domain configured (if applicable)
- [ ] All redirects working (`/papers`, `/github`, `/repo`)
- [ ] Security headers verified
- [ ] Mobile responsiveness tested
- [ ] Automatic deployments confirmed

---

**The LogLine Foundation**  
*Production deployment made simple.*
