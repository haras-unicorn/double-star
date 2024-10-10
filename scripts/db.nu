#!/usr/bin/env nu

let host = (docker container inspect double-star-surrealdb
  | from json).0.NetworkSettings.Networks.double-star-network.IPAddress
let port = 8000
let endpoint = $"ws://($host):($port)" 

def "main host" [] {
  return $host
}

def "main port" [] {
  return $port
}

def --wrapped "main public" [...args] {
  (exec surreal sql
    --hide-welcome
    --pretty
    -u double_star
    -p double_star
    --namespace double_star
    --database public
    -e $endpoint
    ...($args))
}

def --wrapped "main private" [...args] {
  (exec surreal sql
    --hide-welcome
    --pretty
    -u double_star
    -p double_star
    --namespace double_star
    --database private
    -e $endpoint
    ...($args))
}

def "main isready" []  {
  loop {
    try {
      print $"Checking ($endpoint)"
      surreal isready -e $endpoint o+e>| ignore
      break
    } catch {
      sleep 1sec
    }
  }
}

def "main" [] {}
