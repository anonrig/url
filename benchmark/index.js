const {URL: RustURL, URLSearchParams: RustURLSearchParams} = require('url-wasm')

const Benchmarkify = require("benchmarkify");
const benchmark = new Benchmarkify("URL vs. Rust URL", {minSamples: 1000}).printHeader();

const index = benchmark.createSuite('URL')
index.add('URL', () => new URL('https://www.google.com/path/to/something'))
index.add('Rust::URL', () => new RustURL('https://www.google.com/path/to/something'))

const search_params_set = benchmark.createSuite('URLSearchParams.set')
search_params_set.add('URLSearchParams.set', () => {
  let searchParams = new URLSearchParams('hello=world')
  for (let i = 0; i < 100; i++) {
    searchParams.set(`key-${i}`, `value-${i}`)
  }
  return searchParams.toString()
})
search_params_set.add('Rust::URLSearchParams.set', () => {
  let searchParams = new RustURLSearchParams('hello=world')
    for (let i = 0; i < 100; i++) {
      searchParams.set(`key-${i}`, `value-${i}`)
    }
    return searchParams.toString()
})

const search_params_append = benchmark.createSuite('URLSearchParams.append')
search_params_append.add('URLSearchParams.append', () => {
  let searchParams = new URLSearchParams('hello=world')
  for (let i = 0; i < 100; i++) {
    searchParams.append(`key-${i}`, `value-${i}`)
  }
  return searchParams.toString()
})
search_params_append.add('Rust::URLSearchParams.append', () => {
  let searchParams = new RustURLSearchParams('hello=world')
    for (let i = 0; i < 100; i++) {
      searchParams.append(`key-${i}`, `value-${i}`)
    }
    return searchParams.toString()
})

benchmark.run([index, search_params_set, search_params_append])
