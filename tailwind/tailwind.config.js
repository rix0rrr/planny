/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
      'templates/*.html.tera',
      'templates/partials/*.html.tera',
  ],
  input: 'base.css',
  output: 'static/css/styles.css',
  theme: {
    extend: {},
  },
  plugins: [],
}

