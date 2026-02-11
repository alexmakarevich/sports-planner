module.exports = {
  preset: "ts-jest/presets/default-esm",
  testEnvironment: "node",
  extensionsToTreatAsEsm: [".ts"],
  // transformIgnorePatterns: ["node_modules/(?!axios-cookiejar-support)"],
  transformIgnorePatterns: [
    "/node_modules/(?!axios-cookiejar-support|axios|http-cookie-agent).*$",
  ],
  moduleNameMapper: {
    "^axios-cookiejar-support$":
      "<rootDir>/../node_modules/axios-cookiejar-support/dist/index.js",
  },
};
