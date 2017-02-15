        psql whalewolf < create_views.sql
        psql whalewolf < export_all.sql
        psql whalewolf < export_subset.sql
	dot -Tpdf erd.gv > ../data/erd.pdf

