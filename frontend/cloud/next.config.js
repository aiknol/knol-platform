const path = require('path');

const IS_DEV = process.env.NODE_ENV !== 'production';

/** @type {import('next').NextConfig} */
const nextConfig = {
  // Static export is only needed for production builds. In development
  // the full Next.js server is used so that rewrites can proxy API
  // requests, keeping auth cookies on the same origin.
  ...(!IS_DEV && { output: 'export' }),
  trailingSlash: true,
  outputFileTracingRoot: path.join(__dirname),
  ...(IS_DEV && {
    async rewrites() {
      const tenantApiPort = process.env.NEXT_PUBLIC_TENANT_API_PORT || '3002';
      const tenantApiHost = process.env.NEXT_PUBLIC_TENANT_API_HOST || 'localhost';
      const scheme = process.env.NEXT_PUBLIC_URL_SCHEME || 'http';
      const dest = `${scheme}://${tenantApiHost}:${tenantApiPort}`;
      return [
        { source: '/app/:path*', destination: `${dest}/app/:path*` },
        { source: '/health', destination: `${dest}/health` },
      ];
    },
  }),
};

module.exports = nextConfig;
