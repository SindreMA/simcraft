/** @type {import('next').NextConfig} */
const nextConfig = {
  output: process.env.TAURI_BUILD ? "export" : "standalone",
  images: {
    remotePatterns: [
      {
        protocol: "https",
        hostname: "wow.zamimg.com",
        pathname: "/images/wow/icons/**",
      },
    ],
  },
};

export default nextConfig;
