export $(cat env | egrep -v "(^#.*|^$)" | xargs)
