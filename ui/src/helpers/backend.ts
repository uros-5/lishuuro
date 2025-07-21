export function backend(): string {
  const prod = import.meta.env.PROD
  return prod ? import.meta.env.VITE_BACKEND : 'http://localhost:3000'
}

export function frontend(): string {
  const prod = import.meta.env.PROD
  return prod ? import.meta.env.VITE_BACKEND : 'http://localhost:5173'
}

export function wsUrl(): string {
  return backend().replace('http', 'ws')
}

export function newTitle(title: string) {
  document.title = title + ' - lishuuro.org'
}
