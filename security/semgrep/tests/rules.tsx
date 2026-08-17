// TypeScript / TSX semgrep rule test fixtures
import React from 'react';

export function BadComponent({ html }: { html: string }) {
  // ruleid: eury-no-dangerous-html
  return <div dangerouslySetInnerHTML={{ __html: html }} />;
}

export function OkComponent({ text }: { text: string }) {
  // ok: eury-no-dangerous-html
  return <div>{text}</div>;
}

export function badConsoleLog(event: { payload: string }) {
  // ruleid: eury-no-console-log-payload
  console.log('Got payload:', event.payload);
}

export function okConsoleLog(event: { type: string }) {
  // ok: eury-no-console-log-payload
  console.log('Event received:', event.type);
}

export function badEnvRead() {
  // ruleid: eury-no-raw-process-env
  const secret = process.env.DATABASE_URL;
  return secret;
}

export function badSqlInterpolation(db: any, userInput: string) {
  // ruleid: eury-no-raw-sql-interpolation
  return db.$queryRawUnsafe(`SELECT * FROM users WHERE id = '${userInput}'`);
}

export function okSqlParam(db: any, userInput: string) {
  // ok: eury-no-raw-sql-interpolation
  return db.$queryRaw`SELECT * FROM users WHERE id = ${userInput}`;
}
