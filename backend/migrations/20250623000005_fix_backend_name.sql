-- Fix backend name from llama_cpp to llama.cpp
UPDATE test_runs SET backend = 'llama.cpp' WHERE backend = 'llama_cpp';