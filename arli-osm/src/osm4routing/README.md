Taken the implementation from [osm4routing2](https://github.com/Tristramg/osm4routing2)
The plan is to extend it and improve performance:
1. Add turn restrictions
2. Copy some of the tags to the resulting edges
3. Parallelize
4. Reduce number of allocation (e.g. split_ways copies geometry point by point)