require 'json'
require 'socket'
s = TCPSocket.new(ENV['HOST'] || 'critter.levitati.ng', 6606)
s.puts "LOGIN p2pool-webhook"

run ->(env) do
  body = env['rack.input'].gets
  json = JSON.parse body
  s.puts "TITLE: P2Pool webhook"
  s.puts "BODY RST: #{json['type']}"
  s.puts "SEND"

  [200, {}, []]
end
