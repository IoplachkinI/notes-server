mycurl() {
  curl "localhost:4000/notes" -s -o /dev/null 2> /dev/null &
}
export -f mycurl

for i in {1..1000}; do
  mycurl "item_$i" > /dev/null
done
wait # Wait for all background processes to complete
