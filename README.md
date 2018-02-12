 # Hoop
[![Build Status](https://travis-ci.org/Inner-Heaven/libwhisper-rs.svg?branch=master)](https://travis-ci.org/andoriyu/hoop)
[![codecov](https://codecov.io/gh/Inner-Heaven/libwhisper-rs/branch/master/graph/badge.svg)](https://codecov.io/gh/andoriyu/hoop)
[![Crates.io](https://img.shields.io/crates/v/libwhisper.svg)](https://crates.io/crates/hoop)

 Very naive and probably non-perfomant implementation of fixed size circular buffer. The only difference between that one and the many others is:
 this one has double ended non-consuming iterator support.

## Why?
![](http://i0.kym-cdn.com/entries/icons/facebook/000/022/978/yNlQWRM.jpg)
Imagine you have some metrics data coming in and you need to aggregate over it and at msot you have to go 21 items deep. With Vec and VecDeque you will keep moving and/or allocationg things. This buffer allows you to simple keep writting to it and from time to time grab "Last N items" without removing it from buffer.

## Installation
`hoop` is available on crates.io and can be included in your Cargo enabled project like this:

```
[dependencies]
hoop = "0.2.7"
```

## Usage
 ```rust
 let mut buffer = Hoop::with_capacity(4);
 buffer.write('1');
 buffer.write('2');
 buffer.write('3');
 buffer.write('4');
 let mut iter = buffer.iter();
 assert_eq!(Some(&'1'), iter.next());
 assert_eq!(Some(&'4'), iter.next_back());
 assert_eq!(Some(&'2'), iter.next());
 assert_eq!(Some(&'3'), iter.next_back());
 assert_eq!(None, iter.next());
 assert_eq!(None, iter.next_back());
 ```
