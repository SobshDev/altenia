import { useEffect, useState } from 'react';

async function generateFingerprint(): Promise<string> {
  const components = [
    navigator.userAgent,
    navigator.language,
    screen.width,
    screen.height,
    screen.colorDepth,
    new Date().getTimezoneOffset(),
    navigator.hardwareConcurrency || 0,
    navigator.maxTouchPoints || 0,
  ];

  const data = components.join('|');
  const encoder = new TextEncoder();
  const dataBuffer = encoder.encode(data);

  const hashBuffer = await crypto.subtle.digest('SHA-256', dataBuffer);
  const hashArray = Array.from(new Uint8Array(hashBuffer));
  return hashArray.map((b) => b.toString(16).padStart(2, '0')).join('');
}

export function useDeviceFingerprint(): string {
  const [fingerprint, setFingerprint] = useState<string>(() => {
    return localStorage.getItem('deviceFingerprint') || '';
  });

  useEffect(() => {
    if (fingerprint) return;

    generateFingerprint().then((fp) => {
      localStorage.setItem('deviceFingerprint', fp);
      setFingerprint(fp);
    });
  }, [fingerprint]);

  return fingerprint;
}
