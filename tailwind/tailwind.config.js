/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
      'templates/**/*.html.tera',
  ],
  input: 'base.css',
  output: 'static/css/styles.css',
  theme: {
    extend: {},
  },
  plugins: [],
}

